use crate::agent::Agent;
use crate::ai::AiRunConfig;
use crate::exec::DryMode;
use crate::script::rhai_eval;
use crate::support::hbs::hbs_render;
use crate::{Error, Result};
use genai::chat::ChatRequest;
use genai::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::task::JoinSet;
use value_ext::JsonValueExt;

const DEFAULT_CONCURRENCY: usize = 1;

pub async fn run_agent_items(
	client: &Client,
	agent: &Agent,
	items: Option<Vec<Value>>,
	ai_run_config: AiRunConfig,
) -> Result<()> {
	let concurrency = agent.config().items_concurrency().unwrap_or(DEFAULT_CONCURRENCY);

	// -- Print the run info
	let mut genai_infos: Vec<String> = vec![];
	if let Some(temp) = agent.config().temperature() {
		genai_infos.push(format!("temperature: {temp}"));
	}
	let genai_infos = if genai_infos.is_empty() {
		"".to_string()
	} else {
		format!(" ({})", genai_infos.join(", "))
	};

	println!("Running agent command: {}", agent.name());
	println!("                 from: {}", agent.file_path());
	println!("           with model: {}{genai_infos}\n", agent.genai_model());

	// -- Run the before all
	let (items, before_all_data) = if let Some(before_all_script) = agent.before_all_script() {
		let scope_item = json!({
			"items": items.clone(), // clone because item is reused later
		});

		let before_all_res = rhai_eval(before_all_script, Some(scope_item))?;
		match before_all_res {
			Value::Object(mut obj) => {
				let after_all_items = obj.remove("items");
				let items = match after_all_items {
					Some(Value::Array(new_items)) => Some(new_items),
					// if return items: Null, then will be None, which will have one item of Null below
					// > Note to cancel run, we will allow return {_devai_: {action: "skip"}} (not supported for now)
					Some(Value::Null) => None,
					Some(_) => {
						return Err(Error::BeforeAllFailWrongReturn {
						cause: "Before All script block, return `.items` is not type Array, must be an array (even Array of one if one item)".to_string()
					});
					}
					None => items,
				};
				let before_all_data = obj.remove("before_all_data");
				let keys: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
				if !keys.is_empty() {
					return Err(Error::BeforeAllFailWrongReturn {
						cause: format!("Before All script block, can only return '.items' and/or '.before_all_data' but also returned {}", keys.join(", "))
					});
				}
				(items, before_all_data)
			}
			_ => (items, None),
		}
	} else {
		(items, None)
	};

	// Normalize the items, so, if empty, we have one item of value Value::Null
	let items = items.unwrap_or_else(|| vec![Value::Null]);
	let before_all_data = before_all_data.unwrap_or_default();

	let mut join_set = JoinSet::new();
	let mut in_progress = 0;

	// -- Run the items
	for (item_idx, item) in items.into_iter().enumerate() {
		let client_clone = client.clone();
		let agent_clone = agent.clone();
		let before_all_data_clone = before_all_data.clone();

		let ai_run_config_clone = ai_run_config.clone();

		// Spawn tasks up to the concurrency limit
		join_set.spawn(async move {
			run_agent_item(
				item_idx,
				&client_clone,
				&agent_clone,
				before_all_data_clone,
				item,
				&ai_run_config_clone,
			)
			.await
		});

		in_progress += 1;

		// If we've reached the concurrency limit, wait for one task to complete
		if in_progress >= concurrency {
			if let Some(res) = join_set.join_next().await {
				in_progress -= 1;
				match res {
					Ok(result) => {
						result?;
					}
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
				Ok(result) => {
					result?;
				}
				Err(e) => return Err(Error::custom(format!("Error while remaining item. Cause {e}"))),
			}
		}
	}

	Ok(())
}

/// Run the agent for one item
async fn run_agent_item(
	item_idx: usize,
	client: &Client,
	agent: &Agent,
	before_all_data: Value,
	item: impl Serialize,
	ai_run_config: &AiRunConfig,
) -> Result<Value> {
	// -- prepare the scope_item
	let item = serde_json::to_value(item)?;

	// get the eventual "._label" property of the item
	// try to get the path, name
	let label = get_item_label(&item).unwrap_or_else(|| format!("{item_idx}"));
	println!("\n==== Running item: {}", label);

	let data_rhai_scope = json!({
		"item": item.clone(), // clone because item is reused later
		"before_all_data": before_all_data.clone()
	});

	// -- Execute data
	let data = if let Some(data_script) = agent.data_script().as_ref() {
		rhai_eval(data_script, Some(data_rhai_scope))?
	} else {
		Value::Null
	};
	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute genai if we have an instruction
	let inst = hbs_render(agent.inst(), &data_scope)?;

	let is_inst_empty = inst.trim().is_empty();

	// TODO: Might want to handle if no instruction.
	if ai_run_config.verbose() {
		println!("\n-- Instruction:\n\n{inst}\n")
	}

	// if dry_mode req, we stop
	if matches!(ai_run_config.dry_mode(), DryMode::Req) {
		return Ok(Value::Null);
	}

	// Now execute the instruction
	let ai_output = if !is_inst_empty {
		let chat_req = ChatRequest::from_system(inst);

		let chat_res = client
			.exec_chat(agent.genai_model(), chat_req, Some(agent.genai_chat_options()))
			.await?;
		let chat_res_mode_iden = chat_res.model_iden.clone();
		let ai_output = chat_res.content_text_into_string().unwrap_or_default();

		if ai_run_config.verbose() {
			println!(
				"\n-- AI Output (model: {} | adapter: {})\n\n{ai_output}\n",
				chat_res_mode_iden.model_name, chat_res_mode_iden.adapter_kind
			)
		}
		Value::String(ai_output)
	}
	// if we do not have an instruction, just return null
	else {
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
			"before_all_data": before_all_data,
			"ai_output": ai_output,
		});

		rhai_eval(output_script, Some(scope_output))?
	} else {
		ai_output
	};

	// if the response value is a String, then, print it
	if let Some(response_txt) = response_value.as_str() {
		println!("\n-- Agent Output:\n\n{response_txt}");
	}
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

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::agent::AgentDoc;
	use crate::ai::get_genai_client;
	use crate::test_support::default_agent_config_for_test;
	use crate::types::FileRef;
	use simple_fs::SFile;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_agent_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-simple.md")?;
		let agent = doc.into_agent(default_agent_config_for_test())?;

		// -- Execute
		let res = run_agent_item(0, &client, &agent, Value::Null, Value::Null, &AiRunConfig::default()).await?;

		// -- Check
		assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");

		Ok(())
	}

	#[tokio::test]
	async fn test_run_agent_on_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-on-file.md")?;
		let agent = doc.into_agent(default_agent_config_for_test())?;

		// -- Execute
		let on_file = SFile::new("./src/main.rs")?;
		let file_ref = FileRef::from(on_file);

		let run_output = run_agent_item(0, &client, &agent, Value::Null, file_ref, &AiRunConfig::default()).await?;

		// -- Check
		// The output return the {data_path: data.file.path, item_name: item.name}
		assert_eq!(run_output.x_get::<String>("data_path")?, "./src/main.rs");
		assert_eq!(run_output.x_get::<String>("item_name")?, "main.rs");
		let ai_content = run_output.x_get::<String>("ai_content")?;
		assert!(ai_content.len() > 300, "The AI response should have some content");

		Ok(())
	}

	#[tokio::test]
	async fn test_run_agent_before_all() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-before-all.md")?;
		let agent = doc.into_agent(default_agent_config_for_test())?;

		// -- Execute
		let on_file = SFile::new("./src/main.rs")?;
		let file_ref = FileRef::from(on_file);
		let items = vec![serde_json::to_value(file_ref)?];

		run_agent_items(&client, &agent, Some(items), AiRunConfig::default()).await?;

		// -- Check
		// The output return the {data_path: data.file.path, item_name: item.name}
		// assert_eq!(run_output.x_get::<String>("data_path")?, "./src/main.rs");
		// assert_eq!(run_output.x_get::<String>("item_name")?, "main.rs");
		// let ai_content = run_output.x_get::<String>("ai_content")?;
		// assert!(ai_content.len() > 300, "The AI response should have some content");

		Ok(())
	}
}

// endregion: --- Tests
