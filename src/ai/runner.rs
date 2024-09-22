use crate::agent::Agent;
use crate::script::rhai_eval;
use crate::support::hbs::hbs_render;
use crate::{Error, Result};
use genai::chat::ChatRequest;
use genai::{Client, ModelName};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use value_ext::JsonValueExt;

const MODEL: &str = "gpt-4o-mini";

const DEFAULT_CONCURRENCY: usize = 1;

pub async fn run_agent_items(client: Client, agent: Agent, items: Option<Vec<Value>>) -> Result<()> {
	use tokio::task::JoinSet;
	let model_name = agent.config().model_name().ok_or(Error::CannotRunMissingModel)?;
	let model_name = ModelName::from(model_name);

	let concurrency = agent.config().items_concurrency().unwrap_or(DEFAULT_CONCURRENCY);

	println!("Running agent command: {}", agent.name());
	println!("                 from: {}", agent.file_path());
	println!("           with model: {}\n", model_name);

	// -- Get the Items as Vec<Value>
	let items: Vec<Value> = items
		.into_iter()
		.map(|v| serde_json::to_value(v).map_err(Error::custom))
		.collect::<Result<Vec<_>>>()?;

	// If no items, have one with Value::null
	if items.is_empty() {
		run_agent_item(&client, &agent, Value::Null).await?;
	}
	// If we have items, we run them per concurrency rules
	else {
		let mut join_set = JoinSet::new();
		let mut in_progress = 0;

		for (item_idx, item) in items.into_iter().enumerate() {
			let client_clone = client.clone();
			let agent_clone = agent.clone();

			// get the eventual "._label" property of the item
			// try to get the path, name
			let label = get_item_label(&item).unwrap_or_else(|| format!("{item_idx}"));

			println!("Running item: {}", label);

			// Spawn tasks up to the concurrency limit
			join_set.spawn(async move { run_agent_item(&client_clone, &agent_clone, item).await });

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

	Ok(())
}

async fn run_agent_item(client: &Client, agent: &Agent, item: impl Serialize) -> Result<Value> {
	// -- prepare the scope_item
	let item = serde_json::to_value(item)?;
	let scope_item = json!({
		"item": item.clone(), // clone because item is reused later
	});

	// -- Execuated data
	let data = if let Some(data_script) = agent.data_script().as_ref() {
		rhai_eval(data_script, Some(scope_item))?
	} else {
		Value::Null
	};
	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute the handlebars on instruction
	let inst = hbs_render(agent.inst(), &data_scope)?;

	// -- Execute genai
	let chat_req = ChatRequest::from_system(inst);

	let chat_res = client.exec_chat(MODEL, chat_req, None).await?;
	let ai_output = chat_res.content_text_into_string().unwrap_or_default();

	println!("->> ai_output\n{ai_output}\n");

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
		run_agent_item(&client, &agent, Value::Null).await?;

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

		let run_output = run_agent_item(&client, &agent, file_ref).await?;

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
