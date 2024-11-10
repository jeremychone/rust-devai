use crate::agent::Agent;
use crate::hub::get_hub;
use crate::run::literals::Literals;
use crate::run::run_item::run_agent_item;
use crate::run::{RunBaseOptions, Runtime};
use crate::script::devai_custom::{DevaiCustom, FromValue};
use crate::script::rhai_eval;
use crate::support::strings::truncate_with_ellipsis;
use crate::{Error, Result};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::task::JoinSet;
use value_ext::JsonValueExt;

const DEFAULT_CONCURRENCY: usize = 1;

#[derive(Debug, Serialize, Default)]
pub struct RunCommandResponse {
	pub outputs: Option<Vec<Value>>,
	pub after_all: Option<Value>,
}

pub async fn run_command_agent(
	runtime: &Runtime,
	agent: &Agent,
	items: Option<Vec<Value>>,
	run_base_options: &RunBaseOptions,
	return_output_values: bool,
) -> Result<RunCommandResponse> {
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

	let literals = Literals::from_dir_context_and_agent_path(runtime.dir_context(), agent)?;

	// -- Run the before all
	let (items, before_all_data) = if let Some(before_all_script) = agent.before_all_script() {
		let scope_item = json!({
			"items": items.clone(), // clone because item is reused later
			"CTX": literals.to_ctx_value()
		});

		let before_all_res = rhai_eval(runtime.rhai_engine(), before_all_script, Some(scope_item))?;

		match DevaiCustom::from_value(before_all_res)? {
			// it is an skip action
			FromValue::DevaiCustom(DevaiCustom::ActionSkip { reason }) => {
				let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
				hub.publish(format!("-! DevAI Skip items at Before All section{reason_msg}"))
					.await;
				return Ok(RunCommandResponse::default());
			}

			// it is before_all_response
			FromValue::DevaiCustom(DevaiCustom::BeforeAllResponse {
				items: items_override,
				before_all,
			}) => (items_override.or(items), before_all),

			// just plane value
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

	// -- Initialize outputs for capture
	let mut captured_outputs: Option<Vec<(usize, Value)>> =
		if agent.after_all_script().is_some() || return_output_values {
			Some(Vec::new())
		} else {
			None
		};

	// -- Run the items
	for (item_idx, item) in items.clone().into_iter().enumerate() {
		let runtime_clone = runtime.clone();
		let agent_clone = agent.clone();
		let before_all_clone = before_all.clone();
		let literals = literals.clone();

		let base_run_config_clone = run_base_options.clone();

		// Spawn tasks up to the concurrency limit
		join_set.spawn(async move {
			// Execute the command agent (this will perform do Data, Instruction, and Output stages)
			let output = run_command_agent_item(
				item_idx,
				&runtime_clone,
				&agent_clone,
				before_all_clone,
				item,
				&literals,
				&base_run_config_clone,
			)
			.await?;

			// Process the output
			let output = match DevaiCustom::from_value(output)? {
				// if it is a skip, we skip
				FromValue::DevaiCustom(DevaiCustom::ActionSkip { reason }) => {
					let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
					hub.publish(format!("-! DevAI Skip item at Output stage{reason_msg}")).await;
					Value::Null
				}

				// Any other DevaiCustom is not supported at output stage
				FromValue::DevaiCustom(other) => {
					return Err(Error::custom(format!(
						"devai custom '{}' not supported at the Output stage",
						other.as_ref()
					)))
				}

				// Plain value passthrough
				FromValue::OriginalValue(value) => value,
			};

			Ok((item_idx, output))
		});

		in_progress += 1;

		// If we've reached the concurrency limit, wait for one task to complete
		if in_progress >= concurrency {
			if let Some(res) = join_set.join_next().await {
				in_progress -= 1;
				match res {
					Ok(Ok((item_idx, output))) => {
						if let Some(outputs_vec) = &mut captured_outputs {
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
					if let Some(outputs_vec) = &mut captured_outputs {
						outputs_vec.push((item_idx, output));
					}
				}
				Ok(Err(e)) => return Err(e),
				Err(e) => return Err(Error::custom(format!("Error while remaining item. Cause {e}"))),
			}
		}
	}

	// -- Post-process outputs
	let outputs = if let Some(mut captured_outputs) = captured_outputs {
		captured_outputs.sort_by_key(|(idx, _)| *idx);
		Some(captured_outputs.into_iter().map(|(_, v)| v).collect::<Vec<_>>())
	} else {
		None
	};

	// -- Run the after all
	let after_all = if let Some(after_all_script) = agent.after_all_script() {
		let outputs_value = if let Some(outputs) = outputs.as_ref() {
			Value::Array(outputs.clone())
		} else {
			Value::Null
		};

		let scope_item = json!({
			"items": items,
			"outputs": outputs_value, // Will be Value::Null if outputs were not collected
			"before_all": before_all,
			"CTX": literals.to_ctx_value()
		});
		Some(rhai_eval(runtime.rhai_engine(), after_all_script, Some(scope_item))?)
	} else {
		None
	};

	Ok(RunCommandResponse { after_all, outputs })
}

/// Run the command agent item for the run_command_agent_items
/// Not public by design, should be only used in the context of run_command_agent_items
async fn run_command_agent_item(
	item_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	item: impl Serialize,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Value> {
	let hub = get_hub();

	// -- prepare the scope_item
	let item = serde_json::to_value(item)?;
	// get the eventual "._label" property of the item
	// try to get the path, name
	let label = get_item_label(&item).unwrap_or_else(|| format!("item index: {item_idx}"));
	hub.publish(format!("\n==== Running item: {}", label)).await;

	let res_value = run_agent_item(runtime, agent, before_all, &label, item, literals, run_base_options).await?;

	// if the response value is a String, then, print it
	if let Some(response_txt) = res_value.as_str() {
		let short_text = truncate_with_ellipsis(response_txt, 72);
		hub.publish(format!("-> Agent Output: {short_text}")).await;
	}

	hub.publish(format!("-- DONE (item: {})", label)).await;

	Ok(res_value)
}

/// Workaround to expose the run_command_agent_item only for test.
#[cfg(test)]
pub async fn run_command_agent_item_for_test(
	item_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	item: impl Serialize,
	run_base_options: &RunBaseOptions,
) -> Result<Value> {
	let literals = Literals::from_dir_context_and_agent_path(runtime.dir_context(), agent)?;
	run_command_agent_item(item_idx, runtime, agent, before_all, item, &literals, run_base_options).await
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
#[path = "../_tests/tests_run_agent_llm.rs"]
mod tests_run_agent_llm;

#[cfg(test)]
#[path = "../_tests/tests_run_agent_script.rs"]
mod tests_run_agent_script;

// endregion: --- Tests
