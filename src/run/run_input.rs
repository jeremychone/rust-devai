use crate::agent::{Agent, PromptPart};
use crate::hub::get_hub;
use crate::run::literals::Literals;
use crate::run::{DryMode, RunBaseOptions, Runtime};
use crate::script::devai_custom::{DevaiCustom, FromValue};
use crate::script::rhai_eval;
use crate::support::hbs::hbs_render;
use crate::Result;
use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::ModelName;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct AiResponse {
	pub content: Option<String>,
	pub model_name: ModelName,
	pub adapter_kind: AdapterKind,
}

pub enum RunAgentInputResponse {
	AiReponse(AiResponse),
	OutputResponse(Value),
}

impl RunAgentInputResponse {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.as_deref(),
			RunAgentInputResponse::OutputResponse(value) => value.as_str(),
		}
	}

	/// Note: for now, we do like this. Might want to change that.
	/// Note: There is something to do about AI being able to structured output and manage it her
	/// - If AiResposne take the String as value or Null
	/// - If OutputResponse, then, the value is result
	pub fn into_value(self) -> Value {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.into(),
			RunAgentInputResponse::OutputResponse(value) => value,
		}
	}
}

/// Run and agent input for command agent or solo agent.
pub async fn run_agent_input(
	runtime: &Runtime,
	agent: &Agent,
	before_all_result: Value,
	label: &str,
	input: Value,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
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
			return Ok(None);
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
	let mut chat_messages: Vec<ChatMessage> = Vec::new();
	for prompt_part in agent.prompt_parts() {
		let PromptPart { kind, content } = prompt_part;
		let content = hbs_render(content, &data_scope)?;
		// For now, only add if not empty
		if !content.trim().is_empty() {
			chat_messages.push(ChatMessage {
				role: kind.into(),
				content: content.into(),
			})
		}
	}
	// let inst = hbs_render(agent.inst(), &data_scope)?;

	let is_inst_empty = chat_messages.is_empty();

	// TODO: Might want to handle if no instruction.
	if run_base_options.verbose() {
		hub.publish("\n").await;
		for msg in chat_messages.iter() {
			hub.publish(format!(
				"-- {}:\n{}",
				msg.role,
				msg.content.text_as_str().unwrap_or_default()
			))
			.await;
		}
	}

	// if dry_mode req, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Req) {
		return Ok(None);
	}

	// Now execute the instruction
	let ai_response: Option<AiResponse> = if !is_inst_empty {
		// NOTE: Put the instruction as user as with openai o1-... models does not seem to support system.
		let chat_req = ChatRequest::from_messages(chat_messages);

		hub.publish(format!(
			"-> Sending rendered instruction to {} ...",
			agent.genai_model()
		))
		.await;

		let chat_res = client
			.exec_chat(agent.genai_model(), chat_req, Some(agent.genai_chat_options()))
			.await?;

		hub.publish("<- ai_response content received").await;

		let chat_res_mode_iden = chat_res.model_iden.clone();
		let ai_response_content = chat_res.content_text_into_string().unwrap_or_default();

		if run_base_options.verbose() {
			hub.publish(format!(
				"\n-- AI Output (model: {} | adapter: {})\n\n{ai_response_content}\n",
				chat_res_mode_iden.model_name, chat_res_mode_iden.adapter_kind
			))
			.await;
		}

		Some(AiResponse {
			content: Some(ai_response_content),
			model_name: chat_res_mode_iden.model_name,
			adapter_kind: chat_res_mode_iden.adapter_kind,
		})
	}
	// if we do not have an instruction, just return null
	else {
		hub.publish("-! No instruction, skipping genai.").await;
		None
	};

	// -- if dry_mode res, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Res) {
		return Ok(None);
	}

	// -- Exec output
	let res = if let Some(output_script) = agent.output_script() {
		let scope_output = json!({
			"input": input,
			"data": data,
			"before_all": before_all_result,
			"ai_response": ai_response,
			"CTX": literals.to_ctx_value()
		});

		let output_response = rhai_eval(runtime.rhai_engine(), output_script, Some(scope_output))?;
		Some(RunAgentInputResponse::OutputResponse(output_response))
	} else {
		ai_response.map(RunAgentInputResponse::AiReponse)
	};

	Ok(res)
}
