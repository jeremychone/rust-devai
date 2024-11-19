use genai::chat::ChatRole;

#[derive(Debug, Clone)]
pub struct PromptPart {
	#[allow(unused)] // for now
	pub kind: PartKind,
	pub content: String,
}

#[derive(Debug, Clone)]
pub enum PartKind {
	Instruction,
	System,
	Assistant,
}

// region:    --- Froms

impl From<PartKind> for ChatRole {
	fn from(kind: PartKind) -> Self {
		match kind {
			PartKind::Instruction => ChatRole::User,
			PartKind::System => ChatRole::System,
			PartKind::Assistant => ChatRole::Assistant,
		}
	}
}

impl From<&PartKind> for ChatRole {
	fn from(kind: &PartKind) -> Self {
		match kind {
			PartKind::Instruction => ChatRole::User,
			PartKind::System => ChatRole::System,
			PartKind::Assistant => ChatRole::Assistant,
		}
	}
}

// endregion: --- Froms
