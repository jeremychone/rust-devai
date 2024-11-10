pub(super) struct EmbeddedFile {
	pub name: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[
		&EmbeddedFile {
			name: "proof-comments.devai",
			content: include_str!("../../_init/agents/proof-comments.devai"),
		},
		&EmbeddedFile {
			name: "proof-rust-comments.devai",
			content: include_str!("../../_init/agents/proof-rust-comments.devai"),
		},
	]
}

pub(super) fn get_embedded_new_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[&EmbeddedFile {
		name: "default.devai",
		content: include_str!("../../_init/new-command-agent/default.devai"),
	}]
}

pub(super) fn get_embedded_new_solo_agent_files() -> &'static [&'static EmbeddedFile] {
	&[&EmbeddedFile {
		name: "default.devai",
		content: include_str!("../../_init/new-solo-agent/default.devai"),
	}]
}
