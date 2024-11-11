use crate::agent::Agent;
use crate::hub::get_hub;
use crate::run::literals::Literals;
use crate::run::{DryMode, RunBaseOptions, Runtime};
use crate::script::devai_custom::{DevaiCustom, FromValue};
use crate::script::rhai_eval;
use crate::support::hbs::hbs_render;
use crate::Result;
use genai::chat::ChatRequest;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Run and agent input for command agent or solo agent.
pub async fn run_agent_input(
	runtime: &Runtime,
	agent: &Agent,
	before_all_result: Value,
	label: &str,
	input: Value,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Value> {
	let hub = get_hub();
	let client = runtime.genai_client();

	// -- Build the _ctx
	let data_rhai_scope = json!({
		"input": input.clone(), // clone because input is reused later
		"before_all": before_all_result.clone(),
		"CTX": literals.to_ctx_value()
	});

	// -- Execute data
	let data = if let Some(data_script) = agent.data_script().as_ref() {
		rhai_eval(runtime.rhai_engine(), data_script, Some(data_rhai_scope))?
	} else {
		Value::Null
	};

	// skip input if devai action is sent
	let data = match DevaiCustom::from_value(data)? {
		// If it is not a DevaiCustom the data is the orginal value
		FromValue::OriginalValue(data) => data,

		// If we have a skip, we can skip
		FromValue::DevaiCustom(DevaiCustom::Skip { reason }) => {
			let reason_txt = reason.map(|r| format!(" (Reason: {r})")).unwrap_or_default();

			hub.publish(format!("-! DevAI Skip input at Data stage: {label}{reason_txt}"))
				.await;
			return Ok(Value::Null);
		}

		FromValue::DevaiCustom(other) => {
			return Err(format!(
				"-! DevAI Custom '{}' is not supported at the Data stage",
				other.as_ref()
			)
			.into())
		}
	};

	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute genai if we have an instruction
	let inst = hbs_render(agent.inst(), &data_scope)?;

	let is_inst_empty = inst.trim().is_empty();

	// TODO: Might want to handle if no instruction.
	if run_base_options.verbose() {
		hub.publish(format!("\n-- Instruction:\n\n{inst}\n")).await;
	}

	// if dry_mode req, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Req) {
		return Ok(Value::Null);
	}

	// Now execute the instruction
	let ai_output = if !is_inst_empty {
		// NOTE: Put the instruction as user as with openai o1-... models does not seem to support system.
		let chat_req = ChatRequest::from_user(inst);

		hub.publish(format!(
			"-> Sending rendered instruction to {} ...",
			agent.genai_model()
		))
		.await;

		let chat_res = client
			.exec_chat(agent.genai_model(), chat_req, Some(agent.genai_chat_options()))
			.await?;
		hub.publish("<- ai_output received").await;
		let chat_res_mode_iden = chat_res.model_iden.clone();
		let ai_output = chat_res.content_text_into_string().unwrap_or_default();

		if run_base_options.verbose() {
			hub.publish(format!(
				"\n-- AI Output (model: {} | adapter: {})\n\n{ai_output}\n",
				chat_res_mode_iden.model_name, chat_res_mode_iden.adapter_kind
			))
			.await;
		}
		Value::String(ai_output)
	}
	// if we do not have an instruction, just return null
	else {
		hub.publish("-! No instruction, skipping genai.").await;
		Value::Null
	};

	// -- if dry_mode res, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Res) {
		return Ok(Value::Null);
	}

	// -- Exec output
	let response_value: Value = if let Some(output_script) = agent.output_script() {
		let scope_output = json!({
			"input": input,
			"data": data,
			"before_all": before_all_result,
			"ai_output": ai_output,
		});

		rhai_eval(runtime.rhai_engine(), output_script, Some(scope_output))?
	} else {
		ai_output
	};

	Ok(response_value)
}
