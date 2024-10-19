use crate::agent::Agent;
use crate::ai::{AiCommandConfig, AiSoloConfig};
use crate::exec::DryMode;
use crate::hub::get_hub;
use crate::script::devai_custom::{DevaiCustom, FromValue};
use crate::script::rhai_eval;
use crate::support::hbs::hbs_render;
use crate::support::truncate_with_ellipsis;
use crate::types::FileRef;
use crate::{Error, Result};
use genai::chat::ChatRequest;
use genai::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::write;
use tokio::task::JoinSet;
use value_ext::JsonValueExt;

const DEFAULT_CONCURRENCY: usize = 1;

pub async fn run_solo_agent(client: &Client, agent: &Agent, ai_solo_config: AiSoloConfig) -> Result<()> {
	let hub = get_hub();

	// -- Print the run info
	let genai_info = get_genai_info(agent);
	hub.publish(format!(
		"Running solo agent: {}\n        with model: {}{genai_info}",
		agent.file_path(),
		agent.genai_model()
	))
	.await;

	// -- Run the agent
	let label = agent.file_path();
	let item = FileRef::from(ai_solo_config.target_path());
	let item = serde_json::to_value(item)?;
	let before_all_data = Value::Null;
	let res_value = run_agent_item(label, client, agent, before_all_data, item, &(&ai_solo_config).into()).await?;

	if let Value::String(text) = res_value {
		write(ai_solo_config.target_path(), text)?;
		hub.publish(format!(
			"-> Solo Agent ouput saved to: {}",
			ai_solo_config.target_path()
		))
		.await;
	} else {
		hub.publish("-! Solo Agent return not text. Skipping saving to file.").await;
	}

	hub.publish("-- DONE").await;

	Ok(())
}

pub async fn run_command_agent(
	client: &Client,
	agent: &Agent,
	items: Option<Vec<Value>>,
	ai_run_config: AiCommandConfig,
) -> Result<()> {
	let hub = get_hub();
	let concurrency = agent.config().items_concurrency().unwrap_or(DEFAULT_CONCURRENCY);

	// -- Print the run info
	let genai_info = get_genai_info(agent);
	hub.publish(format!(
		"Running agent command: {}\n                 from: {}\n           with model: {}{genai_info}",
		agent.name(),
		agent.file_path(),
		agent.genai_model()
	))
	.await;

	// -- Run the before all
	let (items, before_all_data) = if let Some(before_all_script) = agent.before_all_script() {
		let scope_item = json!({
			"items": items.clone(), // clone because item is reused later
		});

		let before_all_res = rhai_eval(before_all_script, Some(scope_item))?;

		match DevaiCustom::from_value(before_all_res)? {
			FromValue::DevaiCustom(DevaiCustom::ActionSkip { reason }) => {
				let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
				hub.publish(format!("-- DevAI Skip items at Before All section{reason_msg}"))
					.await;
				return Ok(());
			}
			FromValue::DevaiCustom(DevaiCustom::BeforeAll {
				items_override,
				before_all,
			}) => (items_override.or(items), before_all),
			FromValue::OriginalValue(value) => (items, Some(value)),
		}
	} else {
		(items, None)
	};

	// Normalize the items, so, if empty, we have one item of value Value::Null
	let items = items.unwrap_or_else(|| vec![Value::Null]);
	let before_all = before_all_data.unwrap_or_default();

	let mut join_set = JoinSet::new();
	let mut in_progress = 0;

	// -- Initialize outputs capture
	let mut outputs: Option<Vec<(usize, Value)>> = if agent.after_all_script().is_some() {
		Some(Vec::new())
	} else {
		None
	};

	// -- Run the items
	for (item_idx, item) in items.clone().into_iter().enumerate() {
		let client_clone = client.clone();
		let agent_clone = agent.clone();
		let before_all_clone = before_all.clone();

		let ai_run_config_clone = ai_run_config.clone();

		// Spawn tasks up to the concurrency limit
		join_set.spawn(async move {
			let output = run_command_agent_item(
				item_idx,
				&client_clone,
				&agent_clone,
				before_all_clone,
				item,
				&ai_run_config_clone,
			)
			.await?;
			Ok((item_idx, output))
		});

		in_progress += 1;

		// If we've reached the concurrency limit, wait for one task to complete
		if in_progress >= concurrency {
			if let Some(res) = join_set.join_next().await {
				in_progress -= 1;
				match res {
					Ok(Ok((item_idx, output))) => {
						if let Some(outputs_vec) = &mut outputs {
							outputs_vec.push((item_idx, output));
						}
					}
					Ok(Err(e)) => return Err(e),
					Err(e) => return Err(Error::custom(format!("Error while running item. Cause {e}"))),
				}
			}
		}
	}

	// Wait for the remaining tasks to complete
	while in_progress > 0 {
		if let Some(res) = join_set.join_next().await {
			in_progress -= 1;
			match res {
				Ok(Ok((item_idx, output))) => {
					if let Some(outputs_vec) = &mut outputs {
						outputs_vec.push((item_idx, output));
					}
				}
				Ok(Err(e)) => return Err(e),
				Err(e) => return Err(Error::custom(format!("Error while remaining item. Cause {e}"))),
			}
		}
	}

	// -- Post-process outputs
	let outputs_value = if let Some(mut outputs_vec) = outputs {
		outputs_vec.sort_by_key(|(idx, _)| *idx);
		let outputs_values: Vec<Value> = outputs_vec.into_iter().map(|(_, v)| v).collect();
		Value::Array(outputs_values)
	} else {
		Value::Null
	};

	// -- Run the after all
	if let Some(after_all_script) = agent.after_all_script() {
		let scope_item = json!({
			"items": items,
			"outputs": outputs_value, // Will be Value::Null if outputs were not collected
			"before_all": before_all,
		});
		let _after_all_res = rhai_eval(after_all_script, Some(scope_item))?;
	}

	Ok(())
}

/// Run the command agent item for the run_command_agent_items
/// Not public by design, should be only used in the context of run_command_agent_items
async fn run_command_agent_item(
	item_idx: usize,
	client: &Client,
	agent: &Agent,
	before_all: Value,
	item: impl Serialize,
	ai_run_config: &AiCommandConfig,
) -> Result<Value> {
	let hub = get_hub();

	// -- prepare the scope_item
	let item = serde_json::to_value(item)?;
	// get the eventual "._label" property of the item
	// try to get the path, name
	let label = get_item_label(&item).unwrap_or_else(|| format!("item index: {item_idx}"));
	hub.publish(format!("\n==== Running item: {}", label)).await;

	let res_value = run_agent_item(&label, client, agent, before_all, item, ai_run_config).await?;

	// if the response value is a String, then, print it
	if let Some(response_txt) = res_value.as_str() {
		let short_text = truncate_with_ellipsis(response_txt, 72);
		hub.publish(format!("-> Agent Output: {short_text}")).await;
	}

	hub.publish(format!("-- DONE (item: {})", label)).await;

	Ok(res_value)
}

/// Run and agent item for command agent or solo agent.
async fn run_agent_item(
	label: &str,
	client: &Client,
	agent: &Agent,
	before_all_result: Value,
	item: Value,
	ai_run_config: &AiCommandConfig,
) -> Result<Value> {
	let hub = get_hub();

	let data_rhai_scope = json!({
		"item": item.clone(), // clone because item is reused later
		"before_all": before_all_result.clone()
	});

	// -- Execute data
	let data = if let Some(data_script) = agent.data_script().as_ref() {
		rhai_eval(data_script, Some(data_rhai_scope))?
	} else {
		Value::Null
	};

	// skip item if devai action is sent
	let data = match DevaiCustom::from_value(data)? {
		// If it is not a DevaiCustom the data is the orginal value
		FromValue::OriginalValue(data) => data,

		// If we have a skip, we can skip
		FromValue::DevaiCustom(DevaiCustom::ActionSkip { reason }) => {
			let reason_txt = reason.map(|r| format!(" (Reason: {r})")).unwrap_or_default();

			hub.publish(format!("-- DevAI Skip item: {label}{reason_txt}")).await;
			return Ok(Value::Null);
		}

		FromValue::DevaiCustom(other) => {
			return Err(format!(
				"-! DevAI Custom '{}' is not supported at the run agent stage",
				other.as_ref()
			)
			.into())
		}
	};

	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute genai if we have an instruction
	let inst = hbs_render(agent.inst(), &data_scope)?;

	let is_inst_empty = inst.trim().is_empty();

	// TODO: Might want to handle if no instruction.
	if ai_run_config.verbose() {
		hub.publish(format!("\n-- Instruction:\n\n{inst}\n")).await;
	}

	// if dry_mode req, we stop
	if matches!(ai_run_config.dry_mode(), DryMode::Req) {
		return Ok(Value::Null);
	}

	// Now execute the instruction
	let ai_output = if !is_inst_empty {
		// NOTE: Put the instruction as user as with openai o1-... models does not seem to support system.
		let chat_req = ChatRequest::from_user(inst);

		hub.publish(format!(
			"-> Sending rendered instruction to {} ...",
			agent.genai_model()
		))
		.await;

		let chat_res = client
			.exec_chat(agent.genai_model(), chat_req, Some(agent.genai_chat_options()))
			.await?;
		hub.publish("<- ai_output received").await;
		let chat_res_mode_iden = chat_res.model_iden.clone();
		let ai_output = chat_res.content_text_into_string().unwrap_or_default();

		if ai_run_config.verbose() {
			hub.publish(format!(
				"\n-- AI Output (model: {} | adapter: {})\n\n{ai_output}\n",
				chat_res_mode_iden.model_name, chat_res_mode_iden.adapter_kind
			))
			.await;
		}
		Value::String(ai_output)
	}
	// if we do not have an instruction, just return null
	else {
		hub.publish("-! No instruction, skipping genai.").await;
		Value::Null
	};

	// -- if dry_mode res, we stop
	if matches!(ai_run_config.dry_mode(), DryMode::Res) {
		return Ok(Value::Null);
	}

	// -- Exec output
	let response_value: Value = if let Some(output_script) = agent.output_script() {
		let scope_output = json!({
			"item": item,
			"data": data,
			"before_all": before_all_result,
			"ai_output": ai_output,
		});

		rhai_eval(output_script, Some(scope_output))?
	} else {
		ai_output
	};

	Ok(response_value)
}

// region:    --- Support

fn get_item_label(item: &Value) -> Option<String> {
	const LABEL_KEYS: &[&str] = &["path", "name", "label", "_label"];
	for &key in LABEL_KEYS {
		if let Ok(value) = item.x_get::<String>(key) {
			return Some(value);
		}
	}
	None
}

fn get_genai_info(agent: &Agent) -> String {
	let mut genai_infos: Vec<String> = vec![];

	if let Some(temp) = agent.config().temperature() {
		genai_infos.push(format!("temperature: {temp}"));
	}

	if genai_infos.is_empty() {
		"".to_string()
	} else {
		format!(" ({})", genai_infos.join(", "))
	}
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_ai_runner_command.rs"]
mod tests_ai_runner_command;

#[cfg(test)]
#[path = "../_tests/tests_ai_runner_solo.rs"]
mod tests_ai_runner_solo;
// endregion: --- Tests
