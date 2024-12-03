use crate::agent::Agent;
use crate::run::DirContext;
use crate::script::LuaEngine;
use crate::Result;
use serde_json::{Map, Value};
use simple_fs::SPath;
use std::sync::Arc;

/// TODO: Will need to put the Vec in Arc, since this clone what a bit
#[derive(Debug, Default, Clone)]
pub struct Literals {
	/// The store of all literals, pattern and value
	/// e.g. `vec![("&DEVAI_AGENT_DIR","./.devai/custom/command-agent/some.devai")]`
	store: Arc<Vec<(&'static str, String)>>,
}

/// Constructors
impl Literals {
	pub(super) fn from_dir_context_and_agent_path(dir_context: &DirContext, agent: &Agent) -> Result<Literals> {
		// let mut literals = Literals::default();

		let mut store = Vec::new();

		let agent_path = dir_context.current_dir().join(agent.file_path())?;
		let agent_path = agent_path.diff(dir_context.workspace_dir())?;
		// Add back the './' prefix to follow convention of being relative to workspace_dir
		let agent_path = SPath::new(format!("./{agent_path}"))?;

		let agent_dir = agent_path
			.parent()
			.ok_or_else(|| format!("Agent {agent_path} does not have a parent dir"))?;

		let devai_dir = dir_context.devai_dir();

		store.push(("PWD", dir_context.current_dir().to_string()));

		// resolved name from the command
		//   - (so, when pc, it's `proof-comment`)
		//   - When "my-cool-agent/main.md" `my-cool-agent`
		//          agent_name: `my-cool-agent`)
		//   - When "my-cool-agent/cool-specialized.md"
		//          agent_name: `my-cool-agent/cool-specialized`
		//   - When `devai run ./my-folder/command-agent-jc`
		//          agent_name: `./my-folder/command-agent-jc`
		// literals.append("AGENT_NAME", ???);

		// The workspace_dir should be absolute, and all of the other paths will relative to it.
		store.push(("WORKSPACE_DIR", dir_context.workspace_dir().to_string()));

		store.push(("DEVAI_DIR", devai_dir.devai_dir().to_string()));

		store.push(("AGENT_NAME", agent.name().to_string()));
		store.push(("AGENT_FILE_NAME", agent_path.name().to_string()));
		store.push(("AGENT_FILE_PATH", agent_path.to_str().to_string()));
		store.push(("AGENT_FILE_DIR", agent_dir.to_string()));
		store.push(("AGENT_FILE_STEM", agent_path.stem().to_string()));

		Ok(Self { store: Arc::new(store) })
	}
}

/// Getters
impl Literals {
	// pub fn append(&mut self, pattern: impl Into<String>, value: impl Into<String>) {

	// }

	// Your existing add method...
	#[allow(unused)]
	pub fn as_strs(&self) -> Vec<(&str, &str)> {
		self.store.iter().map(|(p, v)| (*p, v.as_str())).collect()
	}
}

/// Transformers
impl Literals {
	pub fn to_ctx_lua_value(&self, lua_engine: &LuaEngine) -> Result<mlua::Value> {
		let table = lua_engine.create_table()?;
		for (name, value) in self.as_strs() {
			table.set(name, value)?;
		}
		Ok(mlua::Value::Table(table))
	}

	#[allow(unused)]
	pub fn to_ctx_value(&self) -> Value {
		let mut _ctx = Map::new();
		for (name, value) in self.as_strs() {
			_ctx.insert(name.to_string(), value.into());
		}
		Value::Object(_ctx)
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;
	use std::path::Path;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_literals_devai_dir() -> Result<()> {
		let script = r#"
return {
	  WORKSPACE_DIR = CTX.WORKSPACE_DIR,
		DEVAI_DIR        = CTX.DEVAI_DIR,
		AGENT_FILE_NAME  = CTX.AGENT_FILE_NAME,
		AGENT_FILE_PATH  = CTX.AGENT_FILE_PATH,
		AGENT_FILE_DIR   = CTX.AGENT_FILE_DIR,
		AGENT_FILE_STEM  = CTX.AGENT_FILE_STEM,
}
		"#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		// workspace_dir
		let workspace_dir = res.x_get_as::<&str>("WORKSPACE_DIR")?;
		assert!(Path::new(workspace_dir).is_absolute(), "workspace_dir must be absolute");
		assert!(
			workspace_dir.ends_with("tests-data/sandbox-01"),
			"WORKSPACE_DIR must end with 'tests-data/sandbox-01'"
		);

		// devai dir
		assert_eq!(res.x_get_as::<&str>("DEVAI_DIR")?, "./.devai");

		assert_eq!(
			res.x_get_as::<&str>("AGENT_FILE_PATH")?,
			"./mock/reflective-agent.devai"
		);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_DIR")?, "./mock");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_NAME")?, "reflective-agent.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_STEM")?, "reflective-agent");

		Ok(())
	}
}

// endregion: --- Tests
