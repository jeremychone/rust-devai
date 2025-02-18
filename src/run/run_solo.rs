use crate::agent::Agent;
use crate::hub::get_hub;
use crate::run::literals::Literals;
use crate::run::run_input::run_agent_input;
use crate::run::support::get_genai_info;
use crate::run::{PathResolver, RunSoloOptions, Runtime};
use crate::types::FileMeta;
use crate::Result;
use serde_json::Value;
use std::fs::write;

pub async fn run_solo_agent(
	runtime: &Runtime,
	agent: &Agent,
	run_solo_options: &RunSoloOptions,
	mode: PathResolver,
) -> Result<()> {
	let hub = get_hub();

	// -- Print the run info
	let genai_info = get_genai_info(agent);
	hub.publish(format!(
		"==== Running solo agent: {}\n        with model: {}{genai_info}",
		agent.file_path(),
		agent.options_as_ref().resolve_model().unwrap_or("No agent") // NOTE: Solo will be removed.
	))
	.await;

	let literals = Literals::from_dir_context_and_agent_path(runtime.dir_context(), agent)?;

	// -- Run the agent
	let label = agent.file_path();
	let input = FileMeta::from(run_solo_options.target_path());
	let input = serde_json::to_value(input)?;
	let before_all_data = Value::Null;
	let agent_options = agent.options_as_ref();

	// NOTE: Starting to deprecate/remove run solo

	todo!()

	// let run_input_response = run_agent_input(
	// 	runtime,
	// 	agent,
	// 	agent_options, // not needed to support agent_options_ov, solo will be deprecated.
	// 	before_all_data,
	// 	label,
	// 	input,
	// 	&literals,
	// 	run_solo_options.base_run_config(),
	// )
	// .await?;

	// let run_input_value = run_input_response.map(|v| v.into_value()).unwrap_or_default();

	// if let Value::String(text) = run_input_value {
	// 	let target_path = run_solo_options.target_path();
	// 	let target_full_path = runtime.dir_context().resolve_path(target_path, mode)?;
	// 	write(target_full_path, text)?;
	// 	hub.publish(format!(
	// 		"-> Solo Agent ouput saved to: {}",
	// 		run_solo_options.target_path()
	// 	))
	// 	.await;
	// } else {
	// 	hub.publish("-! Solo Agent return not text. Skipping saving to file.").await;
	// }

	// hub.publish("-- DONE").await;

	// Ok(())
}

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_run_solo.rs"]
mod tests;

// endregion: --- Tests
