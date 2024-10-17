type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::load_test_agent;
use crate::ai::get_genai_client;
use simple_fs::read_to_string;

#[tokio::test]
async fn test_run_agent_s_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let client = get_genai_client()?;
	let agent = load_test_agent("./tests-data/solo/simple.md.devai")?;
	let fx_target_path = "./tests-data/solo/simple.md";
	let ai_solo_config = AiSoloConfig::from_target_path(fx_target_path)?;

	// -- Execute
	run_solo_agent(&client, &agent, ai_solo_config).await?;

	// -- Check
	// assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");
	let content = read_to_string(fx_target_path)?;
	assert_eq!(content, "Hello from simple.md.devai");

	Ok(())
}
