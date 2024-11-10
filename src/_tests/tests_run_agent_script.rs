type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::{assert_contains, load_inline_agent, load_test_agent, run_test_agent_with_item, HubCapture};
use crate::types::FileRef;
use simple_fs::SPath;

#[tokio::test]
async fn test_run_agent_script_hello_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-hello.md", &runtime)?;

	// -- Execute
	let res = run_test_agent_with_item(&runtime, &agent, "item-01").await?;

	// -- Check
	// Note here '' because item is null
	assert_eq!(
		res.as_str().ok_or("Should have output result")?,
		"hello 'item-01' from agent-hello.md"
	);

	Ok(())
}

/// TODO: This test needs to be fixed. It sometimes fails, which is not an issue (yet) for production.
///       However, when multiple runtimes are used (as is the case for testing), the hub is shared, and the capture might be off.
///       The hub will need to be per runtime, or there should be a way to ensure that all events are sent or something similar.
#[tokio::test]
async fn test_run_agent_script_before_all_simple() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-before-all.md", &runtime)?;
	let hub_capture = HubCapture::new_and_start();

	// -- Execute
	let on_path = SPath::new("./some-random/file.txt")?;
	let path_ref = FileRef::from(on_path);
	let items = vec![serde_json::to_value(path_ref)?];

	let _res = run_command_agent(&runtime, &agent, Some(items), &RunBaseOptions::default(), false).await;

	// -- Check
	let hub_content = hub_capture.into_content().await?;
	assert_contains(
		&hub_content,
		"Agent Output: Some Before All - Some Data - ./some-random/file.txt",
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_before_all_items_reshape() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01()?;
	let agent = load_test_agent("./agent-script/agent-before-all-items-reshape.devai", &runtime)?;
	// let hub_capture = HubCapture::new_and_start();

	// -- Exec
	let items = vec!["one".into(), "two".into()];
	let res = run_command_agent(&runtime, &agent, Some(items), &RunBaseOptions::default(), true)
		.await?
		.ok_or("Should have output values")?;

	// -- Check
	let res = res.iter().map(|v| v.as_str().unwrap_or_default()).collect::<Vec<_>>();
	assert_eq!(res[0], "Data with item: 'one-0'");
	assert_eq!(res[1], "Data with item: 'two-1'");
	assert_eq!(res[2], "Data with item: 'C'");

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
	let fx_items = &["one", "two", "three"];
	let fx_agent = format!(
		r#"
# Data
```rhai
if item == "one" {{
  return devai::action_skip({reason_str});
}}
```

# Output 

```rhai
return "output for: " + item
```
	"#
	);

	let agent = load_inline_agent("./dummy/path.devai", fx_agent)?;

	let hub_capture = HubCapture::new_and_start();

	// -- Execute
	let items = fx_items.iter().map(|v| Value::String(v.to_string())).collect();
	let res = run_command_agent(&runtime, &agent, Some(items), &RunBaseOptions::default(), true)
		.await?
		.ok_or("Should have output result")?;

	// -- Check
	let hub_content = hub_capture.into_content().await?;
	// check the prints/hub:
	assert!(
		hub_content.contains("-! DevAI Skip item: item index: 0"),
		"should have skipped item 0"
	);
	if let Some(reason) = reason.as_ref() {
		assert!(hub_content.contains(reason), "should have reason in the skip message");
	}

	// check the result
	assert_eq!(res.first().ok_or("Should have item 0")?, &Value::Null);
	assert_eq!(
		res.get(1)
			.ok_or("Should have item 1")?
			.as_str()
			.ok_or("item 1 should be string")?,
		"output for: two"
	);
	assert_eq!(
		res.get(2)
			.ok_or("Should have item 2")?
			.as_str()
			.ok_or("item 2 should be string")?,
		"output for: three"
	);

	Ok(())
}
