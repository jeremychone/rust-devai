use crate::run::{run_command_agent_item_for_test, Runtime};
use crate::Result;
use crate::_test_support::load_reflective_agent;
use crate::run::RunBaseOptions;
use serde_json::Value;

/// A reflective agent just return the value of the data rhai section.
/// It's useful for testing rhai module functions.
///
pub async fn run_reflective_agent(data_rhai_code: &str, item: Option<Value>) -> Result<Value> {
	let runtime = Runtime::new_for_test()?;

	let agent = load_reflective_agent(data_rhai_code)?;
	let item = if let Some(item) = item { item } else { Value::Null };

	run_command_agent_item_for_test(0, &runtime, &agent, Value::Null, item, &RunBaseOptions::default()).await
}
