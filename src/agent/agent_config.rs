use crate::Result;
use serde_json::Value;
use value_ext::JsonValueExt;

/// Configuration for the Agent, defined in `.devai/config.toml` and
/// optionally overridden in the `# Config` section of the Command Agent Markdown.
///
/// Note: The values are flattened for simplicity but may be nested in the future.
#[derive(Debug, Clone)]
pub struct AgentConfig {
	// The raw model name of the configuration
	model: Option<String>,

	// Runtime settings
	items_concurrency: Option<u32>,
}

// Getters
impl AgentConfig {
	pub fn model(&self) -> Option<&str> {
		self.model.as_deref()
	}

	pub fn items_concurrency(&self) -> Option<usize> {
		self.items_concurrency.map(|v| v as usize)
	}
}

// Constructors
impl AgentConfig {
	/// Creates a new `AgentConfig` with the specified model name.
	pub fn new(model_name: impl Into<String>) -> Self {
		AgentConfig {
			model: Some(model_name.into()),
			items_concurrency: None,
		}
	}

	/// Creates a new `AgentConfig` from a Value document (either from `cargo.toml` or `# Config` section).
	pub fn from_value(value: Value) -> Result<AgentConfig> {
		let model_name = value.x_get("/genai/model").ok();
		let items_concurrency = value.x_get("/runtime/items_concurrency").ok();

		Ok(AgentConfig {
			model: model_name,
			items_concurrency,
		})
	}

	/// Merges the current config with a new config value, returning the merged `AgentConfig`.
	pub fn merge(self, value: Value) -> Result<AgentConfig> {
		let config_ov = AgentConfig::from_value(value)?;

		Ok(AgentConfig {
			model: config_ov.model.or(self.model),
			items_concurrency: config_ov.items_concurrency.or(self.items_concurrency),
		})
	}
}
