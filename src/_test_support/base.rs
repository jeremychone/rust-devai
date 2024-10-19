use crate::agent::{get_solo_and_target_path, Agent, AgentConfig, AgentDoc};
use crate::ai::AiSoloConfig;
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

pub fn load_test_solo_agent_and_ai_config(path: &str) -> Result<(Agent, AiSoloConfig)> {
	let (fx_solo_path, fx_target_path) = get_solo_and_target_path(path)?;
	let agent = load_test_agent(fx_solo_path.to_str())?;
	let ai_solo_config = AiSoloConfig::from_target_path(fx_target_path.to_str())?;

	Ok((agent, ai_solo_config))
}
