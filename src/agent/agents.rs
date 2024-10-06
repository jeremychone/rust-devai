const AGENT_MD_PROOF_RUST_COMMENTS: &str = include_str!("../../_base/agents/proof-rust-comments.md");

pub(super) struct EmbeddedAgentFile {
	pub name: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_agent_files() -> &'static [&'static EmbeddedAgentFile] {
	&[&EmbeddedAgentFile {
		name: "proof-rust-comments.md",
		content: AGENT_MD_PROOF_RUST_COMMENTS,
	}]
}
