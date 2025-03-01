use crate::Result;
use crate::agent::{Agent, AgentRef};
use crate::dir_context::DirContext;
use crate::script::LuaEngine;
use simple_fs::SPath;
use std::sync::Arc;

/// TODO: Will need to put the Vec in Arc, since this clone what a bit
#[derive(Debug, Default, Clone)]
pub struct Literals {
	/// The store of all literals, pattern and value
	/// e.g. `vec![("&AIPACK_AGENT_DIR","./.aipack/custom/command-agent/some.aipack")]`
	store: Arc<Vec<(&'static str, String)>>,
}

/// Constructors
impl Literals {
	pub(super) fn from_dir_context_and_agent_path(dir_context: &DirContext, agent: &Agent) -> Result<Literals> {
		// let mut literals = Literals::default();

		let mut store = Vec::new();

		let agent_path = dir_context.current_dir().join(agent.file_path())?;
		// Add back the './' prefix to follow convention of being relative to workspace_dir
		let agent_path = SPath::new(format!("./{agent_path}"))?;

		let agent_dir = agent_path
			.parent()
			.ok_or_else(|| format!("Agent {agent_path} does not have a parent dir"))?;

		let aipack_paths = dir_context.aipack_paths();

		store.push(("PWD", dir_context.current_dir().to_string()));

		if let AgentRef::PackRef(pack_ref) = agent.agent_ref() {
			store.push(("PACK_NAMESPACE", pack_ref.namespace().to_string()));
			store.push(("PACK_NAME", pack_ref.name().to_string()));
			if let Some(sub_path) = pack_ref.sub_path.as_deref() {
				store.push(("PACK_SUB_PATH", sub_path.to_string()));
			}
			// This will be `demo@craft/some/path`
			store.push(("PACK_REF", pack_ref.to_string()));
			// This will be `demo@craft`
			store.push(("PACK_IDENTITY", pack_ref.identity().to_string()));

			// This will be the absolute path of `demo@craft`
			store.push(("PACK_DIR", pack_ref.pack_dir.to_string()));
		}

		// The workspace_dir should be absolute, and all of the other paths will relative to it.
		store.push(("WORKSPACE_DIR", dir_context.wks_dir().to_string()));

		// Those are the absolute path for `~/.aipack-base/` and `.aipack/`
		store.push(("WORKSPACE_AIPACK_DIR", aipack_paths.wks_aipack_dir().to_string()));
		store.push(("BASE_AIPACK_DIR", aipack_paths.base_aipack_dir().to_string()));

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
	/// Generate a Lua Value
	/// Note: Similar to into_lua but with no
	pub fn to_lua(&self, lua_engine: &LuaEngine) -> Result<mlua::Value> {
		let table = lua_engine.create_table()?;
		for (name, value) in self.as_strs() {
			table.set(name, value)?;
		}
		Ok(mlua::Value::Table(table))
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_ends_with, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_literals_aipack_dir() -> Result<()> {
		let script = r#"
return {
	  WORKSPACE_DIR         = CTX.WORKSPACE_DIR,
		WORKSPACE_AIPACK_DIR  = CTX.WORKSPACE_AIPACK_DIR,
		BASE_AIPACK_DIR       = CTX.BASE_AIPACK_DIR,
		AGENT_FILE_NAME       = CTX.AGENT_FILE_NAME,
		AGENT_FILE_PATH       = CTX.AGENT_FILE_PATH,
		AGENT_FILE_DIR        = CTX.AGENT_FILE_DIR,
		AGENT_FILE_STEM       = CTX.AGENT_FILE_STEM,
}
		"#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		assert_ends_with(res.x_get_str("WORKSPACE_DIR")?, "tests-data/sandbox-01");
		assert_eq!(res.x_get_str("AGENT_FILE_NAME")?, "reflective-agent.aip");
		assert_eq!(res.x_get_str("AGENT_FILE_STEM")?, "reflective-agent");
		assert_ends_with(res.x_get_str("WORKSPACE_AIPACK_DIR")?, "tests-data/sandbox-01/.aipack");
		assert_ends_with(res.x_get_str("BASE_AIPACK_DIR")?, "tests-data/.aipack-base");
		assert_ends_with(res.x_get_str("AGENT_FILE_PATH")?, "mock/reflective-agent.aip");

		Ok(())
	}
}

// endregion: --- Tests
