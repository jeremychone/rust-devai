use crate::Result;

// region:    --- Agent Types

use genai::chat::ChatMessage;
use simple_fs::{read_to_string, SFile};

#[derive(Debug, Clone, Default)]
pub struct Agent {
	/// The agent instruction
	pub inst: String,
	/// Script
	pub data_script: Option<String>,
	/// Messages
	pub messages: Option<Vec<ChatMessage>>,
	pub output_script: Option<String>,
}

// endregion: --- Agent Types
