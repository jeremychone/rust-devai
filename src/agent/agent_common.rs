// region:    --- Agent Types

use genai::chat::ChatMessage;

#[derive(Debug, Clone, Default)]
pub struct Agent {
	/// The agent's instruction
	pub inst: String,
	/// Script
	pub data_script: Option<String>,
	/// Messages
	#[allow(unused)]
	pub messages: Option<Vec<ChatMessage>>,
	pub output_script: Option<String>,
}

// endregion: --- Agent Types
