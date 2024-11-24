use crate::agent::{find_agent, get_solo_and_target_path, Agent, AgentDoc};
use crate::Result;
use crate::_test_support::default_agent_config_for_test;
use crate::run::{PathResolver, RunSoloOptions, Runtime};

/// Load a Agent form a content.
/// - `path` is just to be used as a path for the agent. Not used to load the content.
pub fn load_inline_agent(path: &str, content: impl Into<String>) -> Result<Agent> {
	let doc = AgentDoc::from_content(path, content)?;
	let agent = doc.into_agent("inline-agent", default_agent_config_for_test())?;
	Ok(agent)
}

pub fn load_test_agent(name: &str, runtime: &Runtime) -> Result<Agent> {
	find_agent(name, runtime.dir_context(), PathResolver::DevaiParentDir)
}

pub fn load_test_solo_agent_and_solo_config(path: &str, runtime: &Runtime) -> Result<(Agent, RunSoloOptions)> {
	let (_, target_path) = get_solo_and_target_path(path)?;

	let agent = load_test_agent(path, runtime)?;

	let solo_config = RunSoloOptions::from_target_path(target_path.to_str())?;
	Ok((agent, solo_config))
}

/// Will create and agent where the `# Output` refect the return of the `# Data`
/// Used to test LUA module functions
pub fn load_reflective_agent(data_lua_code: &str) -> Result<Agent> {
	load_inline_agent(
		"./mock/reflective-agent.devai",
		format!(
			r#"
# Data
```lua
{data_lua_code}
```
# Output
```lua
return data
```
	"#
		),
	)
}
