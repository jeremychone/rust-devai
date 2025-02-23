use crate::_test_support::load_reflective_agent;
use crate::Result;
use crate::agent::Agent;
use crate::run::RunBaseOptions;
use crate::run::{Runtime, run_command_agent_input_for_test};
use serde::Serialize;
use serde_json::Value;

/// A reflective agent just return the value of the data Lua section.
/// It's useful for testing Lua module functions.
///
pub async fn run_reflective_agent(data_lua_code: &str, input: Option<Value>) -> Result<Value> {
	let runtime = Runtime::new_test_runtime_sandbox_01()?;

	let agent = load_reflective_agent(data_lua_code)?;
	let input = if let Some(input) = input { input } else { Value::Null };

	let res =
		run_command_agent_input_for_test(0, &runtime, &agent, Value::Null, input, &RunBaseOptions::default()).await?;
	let res = res.map(|v| v.into_value()).unwrap_or_default();
	Ok(res)
}

pub async fn run_test_agent(runtime: &Runtime, agent: &Agent) -> Result<Value> {
	let res = run_command_agent_input_for_test(0, runtime, agent, Value::Null, Value::Null, &RunBaseOptions::default())
		.await?;
	let res = res.map(|v| v.into_value()).unwrap_or_default();
	Ok(res)
}

pub async fn run_test_agent_with_input(runtime: &Runtime, agent: &Agent, input: impl Serialize) -> Result<Value> {
	let res =
		run_command_agent_input_for_test(0, runtime, agent, Value::Null, input, &RunBaseOptions::default()).await?;
	let res = res.map(|v| v.into_value()).unwrap_or_default();
	Ok(res)
}
