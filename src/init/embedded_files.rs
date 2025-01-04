pub(super) struct EmbeddedFile {
	pub name: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[
		&EmbeddedFile {
			name: "ask-devai.devai",
			content: include_str!("../../_init/agents/ask-devai.devai"),
		},
		&EmbeddedFile {
			name: "proof-read.devai",
			content: include_str!("../../_init/agents/proof-read.devai"),
		},
		&EmbeddedFile {
			name: "proof-comments.devai",
			content: include_str!("../../_init/agents/proof-comments.devai"),
		},
		&EmbeddedFile {
			name: "proof-rs-comments.devai",
			content: include_str!("../../_init/agents/proof-rs-comments.devai"),
		},
	]
}

pub(super) fn get_embedded_new_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[&EmbeddedFile {
		name: "default.devai",
		content: include_str!("../../_init/new-command-agent/default.devai"),
	}]
}

pub(super) fn get_embedded_doc_files() -> &'static [&'static EmbeddedFile] {
	&[
		&EmbeddedFile {
			name: "README.md",
			content: include_str!("../../_init/doc/README.md"),
		},
		&EmbeddedFile {
			name: "lua.md",
			content: include_str!("../../_init/doc/lua.md"),
		},
	]
}
