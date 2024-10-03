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

	if let Some(items) = items {
		let mut join_set = JoinSet::new();
		let mut in_progress = 0;

		for (item_idx, item) in items.into_iter().enumerate() {
			let client_clone = client.clone();
			let agent_clone = agent.clone();
			let ai_run_config_clone = ai_run_config.clone();

			// Spawn tasks up to the concurrency limit
			join_set.spawn(async move {
				run_agent_item(item_idx, &client_clone, &agent_clone, item, &ai_run_config_clone).await
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
	}
	// If no items, have one with Value::null
	else {
		run_agent_item(0, client, agent, Value::Null, &ai_run_config).await?;
	}

	Ok(())
}

/// Run the agent for one item
async fn run_agent_item(
	item_idx: usize,
	client: &Client,
	agent: &Agent,
	item: impl Serialize,
	ai_run_config: &AiRunConfig,
) -> Result<Value> {
	// -- prepare the scope_item
	let item = serde_json::to_value(item)?;

	// get the eventual "._label" property of the item
	// try to get the path, name
	let label = get_item_label(&item).unwrap_or_else(|| format!("{item_idx}"));
	println!("\n==== Running item: {}", label);

	let scope_item = json!({
		"item": item.clone(), // clone because item is reused later
	});

	// -- Execute data
	let data = if let Some(data_script) = agent.data_script().as_ref() {
		rhai_eval(data_script, Some(scope_item))?
	} else {
		Value::Null
	};
	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute the handlebars on instruction
	let inst = hbs_render(agent.inst(), &data_scope)?;

	// -- Execute genai

	if ai_run_config.verbose() {
		println!("\n-- Instruction:\n\n{inst}\n")
	}

	// if dry_mode req, we stop
	if matches!(ai_run_config.dry_mode(), DryMode::Req) {
		return Ok(Value::Null);
	}

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

	// if dry_mode req, we stop
	if matches!(ai_run_config.dry_mode(), DryMode::Res) {
		return Ok(Value::Null);
	}

	let response_value: Value = if let Some(output_script) = agent.output_script() {
		let scope_output = json!({
			"item": item,
			"data": data,
			"ai_output": ai_output,
		});

		rhai_eval(output_script, Some(scope_output))?
	} else {
		ai_output.into()
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
		run_agent_item(0, &client, &agent, Value::Null, &AiRunConfig::default()).await?;

		// -- Check

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

		let run_output = run_agent_item(0, &client, &agent, file_ref, &AiRunConfig::default()).await?;

		// -- Check
		// The output return the {data_path: data.file.path, item_name: item.name}
		assert_eq!(run_output.x_get::<String>("data_path")?, "./src/main.rs");
		assert_eq!(run_output.x_get::<String>("item_name")?, "main.rs");
		let ai_content = run_output.x_get::<String>("ai_content")?;
		assert!(ai_content.len() > 300, "The AI response should have some content");

		Ok(())
	}
}

// endregion: --- Tests
