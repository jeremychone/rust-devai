type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::{assert_contains, load_test_agent, run_test_agent, run_test_agent_with_input};
use crate::types::FileRef;
use simple_fs::SPath;
use value_ext::JsonValueExt;

#[tokio::test]
async fn test_run_agent_llm_c_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-llm/agent-simple.md", &runtime)?;

	// -- Execute
	let res = run_test_agent(&runtime, &agent).await?;

	// -- Check
	assert_contains(res.as_str().ok_or("Should have output result")?, "sky");

	Ok(())
}

/// NOTE: RUN REAL AGENT
#[tokio::test]
async fn test_run_agent_llm_c_on_file_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-llm/agent-on-file.md", &runtime)?;

	// -- Execute
	let on_file = SPath::new("./other/hello.txt")?;
	let file_ref = FileRef::from(on_file);

	let res = run_test_agent_with_input(&runtime, &agent, file_ref).await?;

	// -- Check
	// The output return the {data_path: data.file.path, input_name: input.name}
	assert_eq!(res.x_get_str("data_path")?, "./other/hello.txt");
	assert_eq!(res.x_get_str("input_name")?, "hello.txt");
	let ai_content = res.x_get_str("ai_content")?;
	assert!(ai_content.len() > 10, "The AI response should have some content");
	assert_contains(ai_content, "from the other/hello.txt");

	Ok(())
}
