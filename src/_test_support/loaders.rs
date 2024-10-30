use crate::agent::{Agent, AgentDoc};
use crate::Result;
use crate::_test_support::default_agent_config_for_test;
use crate::support::RunSoloOptions;

/// Load a Agent form a content.
/// - `path` is just to be used as a path for the agent. Not used to load the content.
pub fn load_inline_agent(path: &str, content: impl Into<String>) -> Result<Agent> {
	let doc = AgentDoc::from_content(path, content)?;
	let agent = doc.into_agent(default_agent_config_for_test())?;
	Ok(agent)
}

pub fn load_test_agent(path: &str) -> Result<Agent> {
	let doc = AgentDoc::from_file(path)?;
	let agent = doc.into_agent(default_agent_config_for_test())?;
	Ok(agent)
}

pub fn load_test_solo_agent_and_solo_config(path: &str) -> Result<(Agent, RunSoloOptions)> {
	let solo_config = RunSoloOptions::from_path(path)?;
	let agent = load_test_agent(solo_config.solo_path().to_str())?;

	Ok((agent, solo_config))
}

/// Will create and agent where the `# Output` refect the return of the `# Data`
/// Used to test rhai module functions
pub fn load_reflective_agent(data_rhai_code: &str) -> Result<Agent> {
	load_inline_agent(
		"dummy/path/to/dummy_file.devai",
		format!(
			r#"
# Data
```rhai
{data_rhai_code}
```
# Output
```rhai
return data
```
	"#
		),
	)
}
