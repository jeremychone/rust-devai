use crate::agent::{Agent, AgentConfig, AgentDoc};
use crate::support::RunSoloOptions;
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

pub fn load_test_solo_agent_and_ai_config(path: &str) -> Result<(Agent, RunSoloOptions)> {
	let solo_config = RunSoloOptions::from_path(path)?;
	let agent = load_test_agent(solo_config.solo_path().to_str())?;

	Ok((agent, solo_config))
}
