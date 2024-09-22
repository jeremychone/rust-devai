use crate::agent::agent_config::AgentConfig;
use genai::chat::ChatMessage;
use std::sync::Arc;

/// A sync efficient & friendly Agent containing the AgentInner
#[derive(Debug, Clone)]
pub struct Agent {
	pub inner: Arc<AgentInner>,
}

/// Getters
impl Agent {
	pub fn config(&self) -> &AgentConfig {
		&self.inner.config
	}

	pub fn name(&self) -> &str {
		&self.inner.name
	}

	pub fn file_name(&self) -> &str {
		&self.inner.file_name
	}

	pub fn file_path(&self) -> &str {
		&self.inner.file_path
	}

	pub fn inst(&self) -> &str {
		&self.inner.inst
	}

	pub fn data_script(&self) -> Option<&str> {
		self.inner.data_script.as_deref()
	}

	pub fn messages(&self) -> Option<&[ChatMessage]> {
		self.inner.messages.as_deref()
	}

	pub fn output_script(&self) -> Option<&str> {
		self.inner.output_script.as_deref()
	}
}

// region:    --- AgentInner

/// AgentInner is ok to be public to allow user-code to build Agent simply.
#[derive(Debug, Clone)]
pub struct AgentInner {
	pub config: AgentConfig,

	pub name: String,
	pub file_name: String,
	pub file_path: String,

	/// The agent's instruction
	pub inst: String,
	/// Script
	pub data_script: Option<String>,
	/// Messages
	#[allow(unused)]
	pub messages: Option<Vec<ChatMessage>>,
	pub output_script: Option<String>,
}

impl From<AgentInner> for Agent {
	fn from(inner: AgentInner) -> Self {
		Agent { inner: Arc::new(inner) }
	}
}
// endregion: --- AgentInner
