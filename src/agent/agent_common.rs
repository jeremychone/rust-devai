use crate::agent::agent_config::AgentConfig;
use crate::agent::PromptPart;
use crate::{Error, Result};
use genai::chat::ChatOptions;
use genai::ModelName;
use simple_fs::SPath;
use std::sync::Arc;

/// A sync efficient & friendly Agent containing the AgentInner
#[derive(Debug, Clone)]
pub struct Agent {
	inner: Arc<AgentInner>,
	genai_model: ModelName,
	genai_chat_options: Arc<ChatOptions>,
}

/// Constructor from AgentInner
impl Agent {
	pub(super) fn new(agent_inner: AgentInner) -> Result<Agent> {
		let inner = Arc::new(agent_inner);

		let genai_model = inner.genai_model_name.clone().ok_or_else(|| Error::ModelMissing {
			agent_path: inner.file_path.to_string(),
		})?;

		let mut chat_options = ChatOptions::default();
		if let Some(temp) = inner.config.temperature() {
			chat_options.temperature = Some(temp);
		}

		Ok(Agent {
			inner,
			genai_model,
			genai_chat_options: chat_options.into(),
		})
	}
}

/// Getters
impl Agent {
	pub fn genai_model(&self) -> &ModelName {
		&self.genai_model
	}

	pub fn genai_chat_options(&self) -> &ChatOptions {
		&self.genai_chat_options
	}

	pub fn config(&self) -> &AgentConfig {
		&self.inner.config
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
	pub config: AgentConfig,

	pub name: String,
	pub file_name: String,
	pub file_path: String,

	pub genai_model_name: Option<ModelName>,

	pub before_all_script: Option<String>,

	/// Contains the instruction, system, assistant in order of the file
	pub prompt_parts: Vec<PromptPart>,

	/// Script
	pub data_script: Option<String>,
	pub output_script: Option<String>,
	pub after_all_script: Option<String>,
}

// endregion: --- AgentInner
