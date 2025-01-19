pub(super) struct EmbeddedFile {
	/// Relative path to the agent dir
	pub rel_path: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[
		&EmbeddedFile {
			rel_path: "ask-devai.devai",
			content: include_str!("../../_init/agents/ask-devai.devai"),
		},
		&EmbeddedFile {
			rel_path: "proof-read.devai",
			content: include_str!("../../_init/agents/proof-read.devai"),
		},
		&EmbeddedFile {
			rel_path: "proof-comments.devai",
			content: include_str!("../../_init/agents/proof-comments.devai"),
		},
		&EmbeddedFile {
			rel_path: "proof-rs-comments.devai",
			content: include_str!("../../_init/agents/proof-rs-comments.devai"),
		},
		&EmbeddedFile {
			rel_path: "craft/code.devai",
			content: include_str!("../../_init/agents/craft/code.devai"),
		},
		&EmbeddedFile {
			rel_path: "craft/text.devai",
			content: include_str!("../../_init/agents/craft/text.devai"),
		},
		&EmbeddedFile {
			rel_path: "craft/lua/craft.lua",
			content: include_str!("../../_init/agents/craft/lua/craft.lua"),
		},
	]
}

pub(super) fn get_embedded_new_command_agent_files() -> &'static [&'static EmbeddedFile] {
	&[&EmbeddedFile {
		rel_path: "default.devai",
		content: include_str!("../../_init/new-command-agent/default.devai"),
	}]
}

pub(super) fn get_embedded_doc_files() -> &'static [&'static EmbeddedFile] {
	&[
		&EmbeddedFile {
			rel_path: "README.md",
			content: include_str!("../../_init/doc/README.md"),
		},
		&EmbeddedFile {
			rel_path: "lua.md",
			content: include_str!("../../_init/doc/lua.md"),
		},
	]
}
