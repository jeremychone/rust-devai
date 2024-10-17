type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::agent::AgentDoc;
use crate::ai::get_genai_client;
use crate::test_support::default_agent_config_for_test;
use crate::types::FileRef;
use simple_fs::{read_to_string, SFile};
use value_ext::JsonValueExt;

// region:    --- Command Agent

#[tokio::test]
async fn test_run_agent_c_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let doc = AgentDoc::from_file("./tests-data/agents/agent-simple.md")?;
	let agent = doc.into_agent(default_agent_config_for_test())?;

	// -- Execute
	let res = run_command_agent_item(0, &client, &agent, Value::Null, Value::Null, &AiRunConfig::default()).await?;

	// -- Check
	assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_on_file_ok() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let doc = AgentDoc::from_file("./tests-data/agents/agent-on-file.md")?;
	let agent = doc.into_agent(default_agent_config_for_test())?;

	// -- Execute
	let on_file = SFile::new("./src/main.rs")?;
	let file_ref = FileRef::from(on_file);

	let run_output = run_command_agent_item(0, &client, &agent, Value::Null, file_ref, &AiRunConfig::default()).await?;

	// -- Check
	// The output return the {data_path: data.file.path, item_name: item.name}
	assert_eq!(run_output.x_get::<String>("data_path")?, "./src/main.rs");
	assert_eq!(run_output.x_get::<String>("item_name")?, "main.rs");
	let ai_content = run_output.x_get::<String>("ai_content")?;
	assert!(ai_content.len() > 300, "The AI response should have some content");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_before_all() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let doc = AgentDoc::from_file("./tests-data/agents/agent-before-all.md")?;
	let agent = doc.into_agent(default_agent_config_for_test())?;

	// -- Execute
	let on_file = SFile::new("./src/main.rs")?;
	let file_ref = FileRef::from(on_file);
	let items = vec![serde_json::to_value(file_ref)?];

	run_command_agent(&client, &agent, Some(items), AiRunConfig::default()).await?;

	// -- Check
	// TODO: Need to do the check, but for this, we will need to have the "hub" implemented to get the messages

	Ok(())
}

// endregion: --- Command Agent

#[tokio::test]
async fn test_run_agent_s_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let doc = AgentDoc::from_file("./tests-data/solo/simple.md.devai")?;
	let agent = doc.into_agent(default_agent_config_for_test())?;
	let fx_target_path = "./tests-data/solo/simple.md";
	let ai_solo_config = AiSoloConfig::from_target_path("./tests-data/solo/simple.md")?;

	// -- Execute
	run_solo_agent(&client, &agent, ai_solo_config).await?;

	// -- Check
	// assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");
	let content = read_to_string(fx_target_path)?;
	assert_eq!(content, "Hello from simple.md.devai");

	Ok(())
}
