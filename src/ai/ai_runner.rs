use crate::agent::Agent;
use crate::script::rhai_eval;
use crate::tmpl::hbs_render;
use crate::Result;
use genai::chat::ChatRequest;
use genai::Client;
use serde_json::Value;
use std::collections::HashMap;
use value_ext::JsonValueExt;

const MODEL: &str = "gpt-4o-mini";

pub async fn run_agent(client: &Client, agent: &Agent, scope_value: Option<Value>) -> Result<Value> {
	// -- Get the script data (eval script if present)
	let data = if let Some(data_script) = agent.data_script.as_ref() {
		rhai_eval(data_script, scope_value)?
	} else {
		Value::Null
	};
	let hbs_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute the handlebars on instruction
	let inst = hbs_render(&agent.inst, &hbs_scope)?;

	// -- Exec genai
	let chat_req = ChatRequest::from_system(inst);

	let chat_res = client.exec_chat(MODEL, chat_req, None).await?;
	let ai_output = chat_res.content_text_into_string().unwrap_or_default();

	let response_value: Value = if let Some(output_script) = agent.output_script.as_ref() {
		let mut value = Value::x_new_object();
		value.x_insert("ai_output", ai_output)?;
		value.x_insert("data", data)?;
		rhai_eval(output_script, Some(value))?
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
	use simple_fs::SFile;

	#[tokio::test]
	async fn test_run_agent_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-simple.md")?;
		let agent = doc.into_agent()?;

		// -- Exec
		run_agent(&client, &agent, None).await?;

		// -- Check

		Ok(())
	}

	#[tokio::test]
	async fn test_run_agent_on_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let client = get_genai_client()?;
		let doc = AgentDoc::from_file("./tests-data/agents/agent-on-file.md")?;
		let agent = doc.into_agent()?;

		// -- Exec
		let mut root = Value::x_new_object();
		let on_file = SFile::new("./src/main.rs")?;
		let mut on_file_ref = Value::x_new_object();
		on_file_ref.x_insert("name", on_file.file_name())?;
		on_file_ref.x_insert("path", on_file.path())?;
		on_file_ref.x_insert("stem", on_file.file_stem())?;
		on_file_ref.x_insert("ext", on_file.ext())?;
		root.x_insert("item", on_file_ref)?;

		run_agent(&client, &agent, Some(root)).await?;

		// -- Check

		Ok(())
	}
}

// endregion: --- Tests
