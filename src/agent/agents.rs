const AGENT_MD_PROOF_COMMENTS: &str = include_str!("../../agents/proof-comments.md");

pub(super) struct EmbeddedAgentFile {
	pub name: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_agent_files() -> &'static [&'static EmbeddedAgentFile] {
	&[&EmbeddedAgentFile {
		name: "proof-comments.md",
		content: AGENT_MD_PROOF_COMMENTS,
	}]
}
