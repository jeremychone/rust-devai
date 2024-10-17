use crate::agent::{Agent, AgentConfig, AgentDoc};
use crate::Result;

pub const TEST_MODEL: &str = "gpt-4o-mini";

pub fn default_agent_config_for_test() -> AgentConfig {
	AgentConfig::new(TEST_MODEL)
}

pub fn load_test_agent(path: &str) -> Result<Agent> {
	let doc = AgentDoc::from_file(path)?;
	let agent = doc.into_agent(default_agent_config_for_test())?;
	Ok(agent)
}
