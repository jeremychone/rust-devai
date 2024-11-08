use crate::agent::Agent;
use crate::run::DirContext;
use crate::Result;
use simple_fs::SPath;

#[derive(Debug, Default, Clone)]
pub struct Literals {
	/// The store of all literals, pattern and value
	/// e.g. `vec![("&DEVAI_AGENT_DIR","./.devai/custom/command-agent/some.devai")]`
	store: Vec<(String, String)>,
}

/// Consuming iterator
impl IntoIterator for Literals {
	type Item = (String, String);
	type IntoIter = std::vec::IntoIter<Self::Item>;

	fn into_iter(self) -> Self::IntoIter {
		self.store.into_iter()
	}
}

/// Constructors
impl Literals {
	pub(super) fn from_dir_context_and_agent_path(dir_context: &DirContext, agent: &Agent) -> Result<Literals> {
		let mut literals = Literals::default();

		let agent_path = SPath::new(agent.file_path())?;

		let agent_dir = agent_path
			.parent()
			.ok_or_else(|| format!("Agent with path '{agent_path}' does not have a parent path"))?;

		let devai_dir = dir_context.devai_dir();

		literals.append("PWD", dir_context.current_dir());

		// resolved name from the command
		//   - (so, when pc, it's `proof-comment`)
		//   - When "my-cool-agent/main.md" `my-cool-agent`
		//          agent_name: `my-cool-agent`)
		//   - When "my-cool-agent/cool-specialized.md"
		//          agent_name: `my-cool-agent/cool-specialized`
		//   - When `devai run ./my-folder/command-agent-jc`
		//          agent_name: `./my-folder/command-agent-jc`
		// literals.append("AGENT_NAME", ???);

		literals.append("AGENT_FILE_PATH", agent_path.to_str());
		literals.append("AGENT_FILE_DIR", agent_dir);
		literals.append("AGENT_FILE_NAME", agent_path.name());
		literals.append("AGENT_FILE_STEM", agent_path.stem());
		literals.append("DEVAI_DIR", devai_dir.to_str());
		// the devai_parent_dir should be the one that drives the other relative
		// For example, AGENT_DIR and AGENT_PATH should be path diff with DEVAI_PARENT_DIR
		literals.append("DEVAI_PARENT_DIR", dir_context.devai_parent_dir());

		Ok(literals)
	}
}

/// Getters
impl Literals {
	pub fn append(&mut self, pattern: impl Into<String>, value: impl Into<String>) {
		self.store.push((pattern.into(), value.into()));
	}

	// Your existing add method...
	#[allow(unused)]
	pub fn as_strs(&self) -> Vec<(&str, &str)> {
		self.store.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect()
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_litterals_devai_dir() -> Result<()> {
		let script = r#"
return #{
		AGENT_FILE_PATH: CTX.AGENT_FILE_PATH,
		AGENT_FILE_DIR: CTX.AGENT_FILE_DIR,
		AGENT_FILE_NAME: CTX.AGENT_FILE_NAME,
		AGENT_FILE_STEM: CTX.AGENT_FILE_STEM,
		DEVAI_DIR: CTX.DEVAI_DIR,
		DEVAI_PARENT_DIR: CTX.DEVAI_PARENT_DIR,
}
		"#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		assert_eq!(
			res.x_get_as::<&str>("AGENT_FILE_PATH")?,
			"dummy/path/agent-dir/dummy-agent.devai"
		);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_DIR")?, "dummy/path/agent-dir");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_NAME")?, "dummy-agent.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_STEM")?, "dummy-agent");
		assert_eq!(res.x_get_as::<&str>("DEVAI_DIR")?, "./.devai");
		assert_eq!(res.x_get_as::<&str>("DEVAI_PARENT_DIR")?, "./");

		Ok(())
	}
}

// endregion: --- Tests
