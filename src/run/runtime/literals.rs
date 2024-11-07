use crate::agent::Agent;
use crate::run::DirContext;
use crate::Result;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct Literals {
	/// The store of all literals, pattern and value
	/// e.g. `vec![("&DEVAI_AGENT_DIR","./.devai/custom/command-agent/some.devai")]`
	store: Vec<(String, String)>,
}

/// Constructors
impl Literals {
	#[allow(unused)]
	pub(super) fn new(dir_context: &DirContext, agent_file_path: &str) -> Result<Literals> {
		let mut literals = Literals::default();

		let agent_dir = Path::new(agent_file_path)
			.parent()
			.ok_or_else(|| format!("Agent with path '{agent_file_path}' does not have a parent path"))?
			.to_str()
			.ok_or("File path is not utf8")?;

		let devai_dir = dir_context.devai_dir();

		literals.append("$DEVAI_AGENT_DIR", agent_dir);
		literals.append("$DEVAI_AGENT_PATH", agent_file_path);
		literals.append("$DEVAI_DIR", devai_dir.to_str());
		// TOOD: Need to have a better strategy when parent is none
		literals.append(
			"$DEVAI_PARENT_DIR",
			devai_dir.parent().as_ref().map(|p| p.to_str()).unwrap_or_else(|| ""),
		);

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

#[allow(unused)]
fn build_literals(dir_context: &DirContext, agent: &Agent) -> Result<Literals> {
	let mut literals = Literals::default();

	let agent_path = agent.file_path();
	let agent_dir = Path::new(agent.file_path())
		.parent()
		.ok_or_else(|| format!("Agent with path '{}' does not have a parent path", agent.file_path()))?
		.to_str()
		.ok_or("File path is not utf8")?;

	let devai_dir = dir_context.devai_dir();

	literals.append("$DEVAI_AGENT_DIR", agent_dir);
	literals.append("$DEVAI_AGENT_PATH", agent_path);
	literals.append("$DEVAI_DIR", devai_dir.to_str());
	// TOOD: Need to have a better strategy when parent is none
	literals.append(
		"$DEVAI_PARENT_DIR",
		devai_dir.parent().as_ref().map(|p| p.to_str()).unwrap_or_else(|| ""),
	);

	Ok(literals)
}
