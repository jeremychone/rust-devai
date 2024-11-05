#[derive(Debug, Default, Clone)]
pub struct Literals {
	/// The store of all literals, pattern and value
	/// e.g. `vec![("&DEVAI_AGENT_DIR","./.devai/custom/command-agent/some.devai")]`
	store: Vec<(String, String)>,
}

impl Literals {
	pub fn append(&mut self, pattern: impl Into<String>, value: impl Into<String>) {
		self.store.push((pattern.into(), value.into()));
	}

	// Your existing add method...
	pub fn as_strs(&self) -> Vec<(&str, &str)> {
		self.store.iter().map(|(p, v)| (p.as_str(), v.as_str())).collect()
	}
}
