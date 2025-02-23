use crate::agent::PromptPart;
use crate::agent::agent_options::AgentOptions;
use crate::agent::agent_ref::AgentRef;
use crate::{Error, Result};
use genai::ModelName;
use genai::chat::ChatOptions;
use simple_fs::SPath;
use std::sync::Arc;

/// A sync efficient & friendly Agent containing the AgentInner
#[derive(Debug, Clone)]
pub struct Agent {
	inner: Arc<AgentInner>,
	model: ModelName,
	model_resolved: ModelName,
	agent_options_ov: Option<Arc<AgentOptions>>,
	genai_chat_options: Arc<ChatOptions>,
}

/// Constructor from AgentInner
///
/// TODO: Make it DRYer
impl Agent {
	pub(super) fn new(agent_inner: AgentInner) -> Result<Agent> {
		let inner = Arc::new(agent_inner);

		// -- Build the model and model_resolved
		let model = inner.model_name.clone().ok_or_else(|| Error::ModelMissing {
			agent_path: inner.file_path.to_string(),
		})?;
		let model_resolved = inner.agent_options.resolve_model().map(|v| v.into()).unwrap_or(model.clone());

		let chat_options = ChatOptions::from(&*inner.agent_options);

		Ok(Agent {
			inner,
			model,
			model_resolved,
			agent_options_ov: None,
			genai_chat_options: chat_options.into(),
		})
	}

	pub fn new_merge(&self, options: AgentOptions) -> Result<Agent> {
		let options = self.options().merge_new(options)?;
		let inner = self.inner.clone();

		// -- Build the model and model_resolved
		let model = options.model().map(ModelName::from).ok_or_else(|| Error::ModelMissing {
			agent_path: inner.file_path.to_string(),
		})?;
		let model_resolved = options.resolve_model().map(|v| v.into()).unwrap_or(model.clone());

		// -- Build the genai chat optoins
		let chat_options = ChatOptions::from(&options);

		// -- Returns
		Ok(Agent {
			inner,
			model,
			model_resolved,
			agent_options_ov: Some(Arc::new(options)),
			genai_chat_options: chat_options.into(),
		})
	}
}

/// Getters
impl Agent {
	pub fn model(&self) -> &ModelName {
		&self.model
	}

	pub fn model_resolved(&self) -> &ModelName {
		&self.model_resolved
	}

	pub fn genai_chat_options(&self) -> &ChatOptions {
		&self.genai_chat_options
	}

	pub fn options(&self) -> Arc<AgentOptions> {
		self.agent_options_ov
			.clone()
			.unwrap_or_else(|| self.inner.agent_options.clone())
	}

	pub fn options_as_ref(&self) -> &AgentOptions {
		self.agent_options_ov
			.as_ref()
			.map(|o| o.as_ref())
			.unwrap_or(&self.inner.agent_options)
	}

	pub fn name(&self) -> &str {
		&self.inner.name
	}

	#[allow(unused)]
	pub fn file_name(&self) -> &str {
		&self.inner.file_name
	}

	pub fn file_path(&self) -> &str {
		&self.inner.file_path
	}

	pub fn file_dir(&self) -> Result<SPath> {
		Ok(SPath::new(&self.inner.file_path)?
			.parent()
			.ok_or("Agent does not have a parent dir")?)
	}

	pub fn before_all_script(&self) -> Option<&str> {
		self.inner.before_all_script.as_deref()
	}

	pub fn prompt_parts(&self) -> Vec<&PromptPart> {
		self.inner.prompt_parts.iter().collect()
	}

	pub fn data_script(&self) -> Option<&str> {
		self.inner.data_script.as_deref()
	}

	pub fn output_script(&self) -> Option<&str> {
		self.inner.output_script.as_deref()
	}

	pub fn after_all_script(&self) -> Option<&str> {
		self.inner.after_all_script.as_deref()
	}
}

// region:    --- AgentInner

/// AgentInner is ok to be public to allow user-code to build Agent simply.
#[derive(Debug, Clone)]
pub(super) struct AgentInner {
	pub name: String,

	#[allow(unused)]
	pub agent_ref: AgentRef,

	pub file_name: String,
	pub file_path: String,

	pub agent_options: Arc<AgentOptions>,

	/// The model that came from the options
	pub model_name: Option<ModelName>,

	pub before_all_script: Option<String>,

	/// Contains the instruction, system, assistant in order of the file
	pub prompt_parts: Vec<PromptPart>,

	/// Script
	pub data_script: Option<String>,
	pub output_script: Option<String>,
	pub after_all_script: Option<String>,
}

// endregion: --- AgentInner
