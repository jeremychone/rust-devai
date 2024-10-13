pub(super) struct EmbeddedFile {
	pub name: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[&EmbeddedFile {
		name: "proof-rust-comments.devai",
		content: include_str!("../../_base/agents/proof-rust-comments.devai"),
	}]
}

pub(super) fn get_embedded_new_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[&EmbeddedFile {
		name: "default.devai",
		content: include_str!("../../_base/new-command-agent/default.devai"),
	}]
}
