type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::{assert_contains, load_inline_agent, load_test_agent, run_test_agent_with_input};
use crate::types::FileMeta;
use simple_fs::SPath;

#[tokio::test]
async fn test_run_agent_script_hello_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-hello.devai", &runtime)?;

	// -- Execute
	let res = run_test_agent_with_input(&runtime, &agent, "input-01").await?;

	// -- Check
	// Note here '' because input is null
	assert_eq!(
		res.as_str().ok_or("Should have output result")?,
		"Hello 'input-01' from agent-hello.devai"
	);

	Ok(())
}

/// NOTE: For now disable the HubCapture test see below
///
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_agent_script_before_all_simple() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-before-all.devai", &runtime)?;

	// -- Execute
	let on_path = SPath::new("./some-random/file.txt")?;
	let path_ref = FileMeta::from(on_path);
	let inputs = vec![serde_json::to_value(path_ref)?];

	let res = run_command_agent(&runtime, &agent, Some(inputs), &RunBaseOptions::default(), true).await?;

	// -- Check
	let outputs = res.outputs.ok_or("Should have output values")?;
	assert_eq!(outputs.len(), 1, "should have one and only one output");
	let output = outputs.into_iter().next().ok_or("Should have one output")?;
	assert_eq!(
		output.as_str().ok_or("Output should be string")?,
		"Some Before All - Some Data - ./some-random/file.txt"
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_before_all_inputs_reshape() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-before-all-inputs-reshape.devai", &runtime)?;

	// -- Exec
	let inputs = vec!["one".into(), "two".into()];
	let res = run_command_agent(&runtime, &agent, Some(inputs), &RunBaseOptions::default(), true)
		.await?
		.outputs
		.ok_or("Should have output values")?;

	// -- Check
	let res = res.iter().map(|v| v.as_str().unwrap_or_default()).collect::<Vec<_>>();
	assert_eq!(res[0], "Data with input: 'one-0'");
	assert_eq!(res[1], "Data with input: 'two-1'");
	assert_eq!(res[2], "Data with input: 'C'");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_before_all_inputs_gen() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-before-all-inputs-gen.devai", &runtime)?;

	// -- Exec
	let res = run_command_agent(&runtime, &agent, None, &RunBaseOptions::default(), true).await?;

	// -- Check
	let res_value = serde_json::to_value(res)?;

	// check the null values (because of skip or return)
	assert!(
		matches!(res_value.x_get::<Value>("/outputs/1")?, Value::Null),
		"the 2nd input should be null per agent md"
	);
	assert!(
		matches!(res_value.x_get::<Value>("/outputs/3")?, Value::Null),
		"the 4th input should be null per agent md"
	);
	assert!(
		matches!(res_value.x_get::<Value>("/outputs/4")?, Value::Null),
		"the 5th input should be null per agent md"
	);

	// lazy checks with the json string
	let res_pretty = res_value.x_pretty()?.to_string();
	assert_contains(&res_pretty, r#""data": "Data with input: 'one'""#);
	assert_contains(&res_pretty, r#""rexported_inputs": ["#);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_skip_simple() -> Result<()> {
	common_test_run_agent_script_skip(None).await
}

#[tokio::test]
async fn test_run_agent_script_skip_reason() -> Result<()> {
	common_test_run_agent_script_skip(Some("Some reason")).await
}

async fn common_test_run_agent_script_skip(reason: Option<&str>) -> Result<()> {
	let runtime = Runtime::new_test_runtime_sandbox_01()?;

	let reason_str = reason.map(|v| format!("\"{v}\"")).unwrap_or_default();
	// -- Setup & Fixtures
	let fx_inputs = &["one", "two", "three"];
	let fx_agent = format!(
		r#"
# Data
```lua
if input == "one" then
  return devai.skip({reason_str})
end
```

# Output 

```lua
return "output for: " .. input
```
	"#
	);

	let agent = load_inline_agent("./dummy/path.devai", fx_agent)?;

	// -- Execute
	let inputs = fx_inputs.iter().map(|v| Value::String(v.to_string())).collect();
	let res = run_command_agent(&runtime, &agent, Some(inputs), &RunBaseOptions::default(), true)
		.await?
		.outputs
		.ok_or("Should have output result")?;

	// -- Check
	// check the result
	assert_eq!(res.first().ok_or("Should have input 0")?, &Value::Null);
	assert_eq!(
		res.get(1)
			.ok_or("Should have input 1")?
			.as_str()
			.ok_or("input 1 should be string")?,
		"output for: two"
	);
	assert_eq!(
		res.get(2)
			.ok_or("Should have input 2")?
			.as_str()
			.ok_or("input 2 should be string")?,
		"output for: three"
	);

	Ok(())
}
