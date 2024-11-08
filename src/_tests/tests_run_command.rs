type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::{load_inline_agent, load_test_agent, HubCapture};
use crate::types::FileRef;
use simple_fs::SFile;
use value_ext::JsonValueExt;

#[tokio::test]
async fn test_run_agent_c_simple_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_for_test()?;
	let agent = load_test_agent("./tests-data/agents/agent-simple.md")?;

	// -- Execute
	let res = run_command_agent_item(
		0,
		&runtime,
		&agent,
		Value::Null,
		Value::Null,
		&RunBaseOptions::default(),
	)
	.await?;

	// -- Check
	assert_eq!(res.as_str().ok_or("Should have output result")?, "./src/main.rs");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_hello_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_for_test()?;
	let agent = load_test_agent("./tests-data/agents/agent-hello.md")?;

	// -- Execute
	let res = run_command_agent_item(
		0,
		&runtime,
		&agent,
		Value::Null,
		Value::Null,
		&RunBaseOptions::default(),
	)
	.await?;

	// -- Check
	// Note here '' because item is null
	assert_eq!(
		res.as_str().ok_or("Should have output result")?,
		"hello '' from agent-hello.md"
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_on_file_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_for_test()?;
	let agent = load_test_agent("./tests-data/agents/agent-on-file.md")?;

	// -- Execute
	let on_file = SFile::new("./src/main.rs")?;
	let file_ref = FileRef::from(on_file);

	let run_output =
		run_command_agent_item(0, &runtime, &agent, Value::Null, file_ref, &RunBaseOptions::default()).await?;

	// -- Check
	// The output return the {data_path: data.file.path, item_name: item.name}
	assert_eq!(run_output.x_get::<String>("data_path")?, "./src/main.rs");
	assert_eq!(run_output.x_get::<String>("item_name")?, "main.rs");
	let ai_content = run_output.x_get::<String>("ai_content")?;
	assert!(ai_content.len() > 300, "The AI response should have some content");

	Ok(())
}

// #[tokio::test]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_agent_c_before_all_simple() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_for_test()?;
	let agent = load_test_agent("./tests-data/agents/agent-before-all.md")?;
	let hub_capture = HubCapture::new_and_start();

	// -- Execute
	let on_file = SFile::new("./src/main.rs")?;
	let file_ref = FileRef::from(on_file);
	let items = vec![serde_json::to_value(file_ref)?];

	let _res = run_command_agent(&runtime, &agent, Some(items), &RunBaseOptions::default(), false).await;

	// -- Check
	let hub_content = hub_capture.into_content().await?;
	assert!(
		hub_content.contains("-> Agent Output: Some Before All - Some Data - ./src/main.rs"),
		"Agent Output not matching!"
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_c_skip_simple() -> Result<()> {
	common_test_run_agent_c_skip(None).await
}

#[tokio::test]
async fn test_run_agent_c_skip_reason() -> Result<()> {
	common_test_run_agent_c_skip(Some("Some reason")).await
}

async fn common_test_run_agent_c_skip(reason: Option<&str>) -> Result<()> {
	let runtime = Runtime::new_for_test()?;

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
