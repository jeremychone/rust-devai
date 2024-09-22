use crate::agent::AgentConfig;

pub const TEST_MODEL: &str = "gpt-4o-mini";

pub fn default_agent_config_for_test() -> AgentConfig {
	AgentConfig::new(TEST_MODEL)
}
