use crate::ai::{get_genai_client, run_command_agent_item_for_test};
use crate::Result;
use crate::_test_support::load_reflective_agent;
use crate::support::RunBaseOptions;
use serde_json::Value;

/// A reflective agent just return the value of the data rhai section.
/// It's useful for testing rhai module functions.
///
/// Note: This will run a
pub async fn run_reflective_agent(data_rhai_code: &str) -> Result<Value> {
	let client = get_genai_client()?;
	let agent = load_reflective_agent(data_rhai_code)?;

	run_command_agent_item_for_test(0, &client, &agent, Value::Null, Value::Null, &RunBaseOptions::default()).await
}
