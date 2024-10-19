type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::{load_test_agent, HubCapture};
use crate::ai::get_genai_client;
use crate::types::FileRef;
use simple_fs::SFile;
use value_ext::JsonValueExt;

#[tokio::test]
async fn test_run_agent_c_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let agent = load_test_agent("./tests-data/agents/agent-simple.md")?;

	// -- Execute
	let res = run_command_agent_item(0, &client, &agent, Value::Null, Value::Null, &RunBaseOptions::default()).await?;

	// -- Check
	assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_on_file_ok() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let agent = load_test_agent("./tests-data/agents/agent-on-file.md")?;

	// -- Execute
	let on_file = SFile::new("./src/main.rs")?;
	let file_ref = FileRef::from(on_file);

	let run_output =
		run_command_agent_item(0, &client, &agent, Value::Null, file_ref, &RunBaseOptions::default()).await?;

	// -- Check
	// The output return the {data_path: data.file.path, item_name: item.name}
	assert_eq!(run_output.x_get::<String>("data_path")?, "./src/main.rs");
	assert_eq!(run_output.x_get::<String>("item_name")?, "main.rs");
	let ai_content = run_output.x_get::<String>("ai_content")?;
	assert!(ai_content.len() > 300, "The AI response should have some content");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_before_all_simple() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let agent = load_test_agent("./tests-data/agents/agent-before-all.md")?;
	let hub_capture = HubCapture::new_and_start();

	// -- Execute
	let on_file = SFile::new("./src/main.rs")?;
	let file_ref = FileRef::from(on_file);
	let items = vec![serde_json::to_value(file_ref)?];

	run_command_agent(&client, &agent, Some(items), &RunBaseOptions::default()).await?;

	// -- Check
	let hub_content = hub_capture.into_content().await?;
	assert!(
		hub_content.contains("-> Agent Output: Some Before All - Some Data - ./src/main.rs"),
		"Agent Output not matching!"
	);

	Ok(())
}
