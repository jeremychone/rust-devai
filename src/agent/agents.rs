use crate::{Error, Result};
// get the include for the agents/proof-comments

const AGENT_MD_PROOF_COMMENTS: &str = include_str!("../../agents/proof-comments.md");

pub fn get_agent_instruction(name: &str) -> Result<String> {
	match name {
		"pc" | "proof-comments" => Ok(AGENT_MD_PROOF_COMMENTS.to_string()),
		_ => Err(Error::AgentNotFound(name.to_string())),
	}
}
