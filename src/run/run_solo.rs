use crate::agent::Agent;
use crate::hub::get_hub;
use crate::run::run_item::run_agent_item;
use crate::run::support::get_genai_info;
use crate::run::{RunSoloOptions, Runtime};
use crate::types::FileRef;
use crate::Result;
use serde_json::Value;
use std::fs::write;

pub async fn run_solo_agent(runtime: &Runtime, agent: &Agent, run_solo_options: &RunSoloOptions) -> Result<()> {
	let hub = get_hub();

	// -- Print the run info
	let genai_info = get_genai_info(agent);
	hub.publish(format!(
		"==== Running solo agent: {}\n        with model: {}{genai_info}",
		agent.file_path(),
		agent.genai_model()
	))
	.await;

	// -- Run the agent
	let label = agent.file_path();
	let item = FileRef::from(run_solo_options.target_path());
	let item = serde_json::to_value(item)?;
	let before_all_data = Value::Null;
	let res_value = run_agent_item(
		runtime,
		agent,
		before_all_data,
		label,
		item,
		run_solo_options.base_run_config(),
	)
	.await?;

	if let Value::String(text) = res_value {
		write(run_solo_options.target_path(), text)?;
		hub.publish(format!(
			"-> Solo Agent ouput saved to: {}",
			run_solo_options.target_path()
		))
		.await;
	} else {
		hub.publish("-! Solo Agent return not text. Skipping saving to file.").await;
	}

	hub.publish("-- DONE").await;

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_run_solo.rs"]
mod tests;

// endregion: --- Tests
