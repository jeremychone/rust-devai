use crate::agent::Agent;
use crate::script::rhai_eval;
use crate::tmpl::hbs_render;
use crate::Result;
use genai::chat::ChatRequest;
use genai::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

const MODEL: &str = "gpt-4o-mini";

pub async fn run_agent(client: &Client, agent: &Agent, item: impl Serialize) -> Result<Value> {
	// -- prepare the scope_item
	let item = serde_json::to_value(item)?;
	let scope_item = json!({
		"item": item.clone(), // clone because item is reused later
	});

	// -- Execuated data
	let data = if let Some(data_script) = agent.data_script.as_ref() {
		rhai_eval(data_script, Some(scope_item))?
	} else {
		Value::Null
	};
	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute the handlebars on instruction
	let inst = hbs_render(&agent.inst, &data_scope)?;

	// -- Execute genai
	let chat_req = ChatRequest::from_system(inst);

	let chat_res = client.exec_chat(MODEL, chat_req, None).await?;
	let ai_output = chat_res.content_text_into_string().unwrap_or_default();

	let response_value: Value = if let Some(output_script) = agent.output_script.as_ref() {
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

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::agent::AgentDoc;
	use crate::ai::get_genai_client;
	use crate::types::FileRef;
	use simple_fs::SFile;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_agent_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-simple.md")?;
		let agent = doc.into_agent()?;

		// -- Execute
		run_agent(&client, &agent, Value::Null).await?;

		// -- Check

		Ok(())
	}

	#[tokio::test]
	async fn test_run_agent_on_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-on-file.md")?;
		let agent = doc.into_agent()?;

		// -- Execute
		let on_file = SFile::new("./src/main.rs")?;
		let file_ref = FileRef::from(on_file);

		let run_output = run_agent(&client, &agent, file_ref).await?;

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
