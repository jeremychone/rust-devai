use crate::agent::Agent;
use crate::hub::get_hub;
use crate::run::literals::Literals;
use crate::run::run_input::run_agent_input;
use crate::run::{RunBaseOptions, Runtime};
use crate::script::devai_custom::{DevaiCustom, FromValue};
use crate::script::rhai_eval;
use crate::support::strings::truncate_with_ellipsis;
use crate::{Error, Result};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::task::JoinSet;
use value_ext::JsonValueExt;

const DEFAULT_CONCURRENCY: usize = 1;

#[derive(Debug, Serialize, Default)]
pub struct RunCommandResponse {
	pub outputs: Option<Vec<Value>>,
	pub after_all: Option<Value>,
}

pub async fn run_command_agent(
	runtime: &Runtime,
	agent: &Agent,
	inputs: Option<Vec<Value>>,
	run_base_options: &RunBaseOptions,
	return_output_values: bool,
) -> Result<RunCommandResponse> {
	let hub = get_hub();
	let concurrency = agent.config().inputs_concurrency().unwrap_or(DEFAULT_CONCURRENCY);

	// -- Print the run info
	let genai_info = get_genai_info(agent);
	hub.publish(format!(
		"Running agent command: {}\n                 from: {}\n           with model: {}{genai_info}",
		agent.name(),
		agent.file_path(),
		agent.genai_model()
	))
	.await;

	let literals = Literals::from_dir_context_and_agent_path(runtime.dir_context(), agent)?;

	// -- Run the before all
	let (inputs, before_all_response) = if let Some(before_all_script) = agent.before_all_script() {
		let before_all_scope = json!({
			"inputs": inputs.clone(), // clone because input is reused later
			"CTX": literals.to_ctx_value()
		});

		let before_all_res = rhai_eval(runtime.rhai_engine(), before_all_script, Some(before_all_scope))?;

		match DevaiCustom::from_value(before_all_res)? {
			// it is an skip action
			FromValue::DevaiCustom(DevaiCustom::Skip { reason }) => {
				let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
				hub.publish(format!("-! DevAI Skip inputs at Before All section{reason_msg}"))
					.await;
				return Ok(RunCommandResponse::default());
			}

			// it is before_all_response
			FromValue::DevaiCustom(DevaiCustom::BeforeAllResponse {
				inputs: inputs_override,
				before_all,
			}) => (inputs_override.or(inputs), before_all),

			// just plane value
			FromValue::OriginalValue(value) => (inputs, Some(value)),
		}
	} else {
		(inputs, None)
	};

	// Normalize the inputs, so, if empty, we have one input of value Value::Null
	let inputs = inputs.unwrap_or_else(|| vec![Value::Null]);
	let before_all = before_all_response.unwrap_or_default();

	let mut join_set = JoinSet::new();
	let mut in_progress = 0;

	// -- Initialize outputs for capture
	let mut captured_outputs: Option<Vec<(usize, Value)>> =
		if agent.after_all_script().is_some() || return_output_values {
			Some(Vec::new())
		} else {
			None
		};

	// -- Run the inputs
	for (input_idx, input) in inputs.clone().into_iter().enumerate() {
		let runtime_clone = runtime.clone();
		let agent_clone = agent.clone();
		let before_all_clone = before_all.clone();
		let literals = literals.clone();

		let base_run_config_clone = run_base_options.clone();

		// Spawn tasks up to the concurrency limit
		join_set.spawn(async move {
			// Execute the command agent (this will perform do Data, Instruction, and Output stages)
			let output = run_command_agent_input(
				input_idx,
				&runtime_clone,
				&agent_clone,
				before_all_clone,
				input,
				&literals,
				&base_run_config_clone,
			)
			.await?;

			// Process the output
			let output = match DevaiCustom::from_value(output)? {
				// if it is a skip, we skip
				FromValue::DevaiCustom(DevaiCustom::Skip { reason }) => {
					let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
					hub.publish(format!("-! DevAI Skip input at Output stage{reason_msg}")).await;
					Value::Null
				}

				// Any other DevaiCustom is not supported at output stage
				FromValue::DevaiCustom(other) => {
					return Err(Error::custom(format!(
						"devai custom '{}' not supported at the Output stage",
						other.as_ref()
					)))
				}

				// Plain value passthrough
				FromValue::OriginalValue(value) => value,
			};

			Ok((input_idx, output))
		});

		in_progress += 1;

		// If we've reached the concurrency limit, wait for one task to complete
		if in_progress >= concurrency {
			if let Some(res) = join_set.join_next().await {
				in_progress -= 1;
				match res {
					Ok(Ok((input_idx, output))) => {
						if let Some(outputs_vec) = &mut captured_outputs {
							outputs_vec.push((input_idx, output));
						}
					}
					Ok(Err(e)) => return Err(e),
					Err(e) => return Err(Error::custom(format!("Error while running input. Cause {e}"))),
				}
			}
		}
	}

	// Wait for the remaining tasks to complete
	while in_progress > 0 {
		if let Some(res) = join_set.join_next().await {
			in_progress -= 1;
			match res {
				Ok(Ok((input_idx, output))) => {
					if let Some(outputs_vec) = &mut captured_outputs {
						outputs_vec.push((input_idx, output));
					}
				}
				Ok(Err(e)) => return Err(e),
				Err(e) => return Err(Error::custom(format!("Error while remaining input. Cause {e}"))),
			}
		}
	}

	// -- Post-process outputs
	let outputs = if let Some(mut captured_outputs) = captured_outputs {
		captured_outputs.sort_by_key(|(idx, _)| *idx);
		Some(captured_outputs.into_iter().map(|(_, v)| v).collect::<Vec<_>>())
	} else {
		None
	};

	// -- Run the after all
	let after_all = if let Some(after_all_script) = agent.after_all_script() {
		let outputs_value = if let Some(outputs) = outputs.as_ref() {
			Value::Array(outputs.clone())
		} else {
			Value::Null
		};

		let after_all_scope = json!({
			"inputs": inputs,
			"outputs": outputs_value, // Will be Value::Null if outputs were not collected
			"before_all": before_all,
			"CTX": literals.to_ctx_value()
		});
		Some(rhai_eval(
			runtime.rhai_engine(),
			after_all_script,
			Some(after_all_scope),
		)?)
	} else {
		None
	};

	Ok(RunCommandResponse { after_all, outputs })
}

/// Run the command agent input for the run_command_agent_inputs
/// Not public by design, should be only used in the context of run_command_agent_inputs
async fn run_command_agent_input(
	input_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	input: impl Serialize,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Value> {
	let hub = get_hub();

	// -- prepare the scope_input
	let input = serde_json::to_value(input)?;
	// get the eventual "._label" property of the input
	// try to get the path, name
	let label = get_input_label(&input).unwrap_or_else(|| format!("input index: {input_idx}"));
	hub.publish(format!("\n==== Running input: {}", label)).await;

	let res_value = run_agent_input(runtime, agent, before_all, &label, input, literals, run_base_options).await?;

	// if the response value is a String, then, print it
	if let Some(response_txt) = res_value.as_str() {
		let short_text = truncate_with_ellipsis(response_txt, 72);
		hub.publish(format!("-> Agent Output: {short_text}")).await;
	}

	hub.publish(format!("-- DONE (input: {})", label)).await;

	Ok(res_value)
}

/// Workaround to expose the run_command_agent_input only for test.
#[cfg(test)]
pub async fn run_command_agent_input_for_test(
	input_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	input: impl Serialize,
	run_base_options: &RunBaseOptions,
) -> Result<Value> {
	let literals = Literals::from_dir_context_and_agent_path(runtime.dir_context(), agent)?;
	run_command_agent_input(
		input_idx,
		runtime,
		agent,
		before_all,
		input,
		&literals,
		run_base_options,
	)
	.await
}

// region:    --- Support

fn get_input_label(input: &Value) -> Option<String> {
	const LABEL_KEYS: &[&str] = &["path", "name", "label", "_label"];
	for &key in LABEL_KEYS {
		if let Ok(value) = input.x_get::<String>(key) {
			return Some(value);
		}
	}
	None
}

fn get_genai_info(agent: &Agent) -> String {
	let mut genai_infos: Vec<String> = vec![];

	if let Some(temp) = agent.config().temperature() {
		genai_infos.push(format!("temperature: {temp}"));
	}

	if genai_infos.is_empty() {
		"".to_string()
	} else {
		format!(" ({})", genai_infos.join(", "))
	}
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_run_agent_llm.rs"]
mod tests_run_agent_llm;

#[cfg(test)]
#[path = "../_tests/tests_run_agent_script.rs"]
mod tests_run_agent_script;

// endregion: --- Tests
