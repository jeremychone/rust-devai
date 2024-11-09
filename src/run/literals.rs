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

		let agent_path = dir_context.current_dir().join(agent.file_path())?;
		let agent_path = agent_path.diff(dir_context.devai_parent_dir())?;
		// Add back the './' prefix to follow convention of being relative to devai_parent_dir
		let agent_path = SPath::new(format!("./{agent_path}"))?;

		let agent_dir = agent_path
			.parent()
			.ok_or_else(|| format!("Agent {agent_path} does not have a parent dir"))?;

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

		// The devai_parent_dir should be absolute, and all of the other paths will relative to it.
		literals.append("DEVAI_PARENT_DIR", dir_context.devai_parent_dir());

		literals.append("DEVAI_DIR", devai_dir.devai_dir());

		literals.append("AGENT_NAME", agent.name());
		literals.append("AGENT_FILE_NAME", agent_path.name());
		literals.append("AGENT_FILE_PATH", agent_path.to_str());
		literals.append("AGENT_FILE_DIR", agent_dir);
		literals.append("AGENT_FILE_STEM", agent_path.stem());

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
	use std::path::Path;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_run_literals_devai_dir() -> Result<()> {
		let script = r#"
return #{
	  DEVAI_PARENT_DIR: CTX.DEVAI_PARENT_DIR,
		DEVAI_DIR:        CTX.DEVAI_DIR,
		AGENT_FILE_NAME:  CTX.AGENT_FILE_NAME,
		AGENT_FILE_PATH:  CTX.AGENT_FILE_PATH,
		AGENT_FILE_DIR:   CTX.AGENT_FILE_DIR,
		AGENT_FILE_STEM:  CTX.AGENT_FILE_STEM,
}
		"#;

		// -- Exec
		let res = run_reflective_agent(script, None).await?;

		// -- Check
		// devai_parent_dir
		let devai_parent_dir = res.x_get_as::<&str>("DEVAI_PARENT_DIR")?;
		assert!(
			Path::new(devai_parent_dir).is_absolute(),
			"devai_parent_dir must be absolute"
		);
		assert!(
			devai_parent_dir.ends_with("tests-data/sandbox-01"),
			"DEVAI_PARENT_DIR must end with 'tests-data/sandbox-01'"
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
