type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::load_test_solo_agent_and_solo_config;

#[tokio::test]
async fn test_run_agent_s_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let (agent, solo_config) = load_test_solo_agent_and_solo_config("./solo/simple.md.devai", &runtime)?;

	// -- Execute
	run_solo_agent(&runtime, &agent, &solo_config, PathResolver::CurrentDir).await?;

	// -- Check
	// assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");
	// let content = read_to_string(solo_config.target_path())?;
	// assert_eq!(
	// 	content,
	// 	"Output - ./tests-data/solo/simple.md - From Data (input.path: ./tests-data/solo/simple.md)"
	// );

	Ok(())
}
