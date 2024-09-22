use crate::Result;
use serde_json::Value;
use value_ext::JsonValueExt;

/// Configuration for the Agent, defined in `.devai/config.toml` and
/// optionally overridden in the `# Config` section of the Command Agent Markdown.
///
/// Note: The values are flattened for simplicity but may be nested in the future.
#[derive(Debug, Clone)]
pub struct AgentConfig {
	// Genai configuration
	model_name: Option<String>,

	// Runtime settings
	items_concurrency: Option<u32>,
}

// Getters
impl AgentConfig {
	pub fn model_name(&self) -> Option<&str> {
		self.model_name.as_deref()
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
			model_name: Some(model_name.into()),
			items_concurrency: None,
		}
	}

	/// Creates a new `AgentConfig` from a Value document (either from `cargo.toml` or `# Config` section).
	pub fn from_value(value: Value) -> Result<AgentConfig> {
		let model_name = value.x_get("/genai/model").ok();
		let items_concurrency = value.x_get("/runtime/items_concurrency").ok();

		Ok(AgentConfig {
			model_name,
			items_concurrency,
		})
	}

	/// Merges the current config with a new config value, returning the merged `AgentConfig`.
	pub fn merge(self, value: Value) -> Result<AgentConfig> {
		let config_ov = AgentConfig::from_value(value)?;

		Ok(AgentConfig {
			model_name: config_ov.model_name.or(self.model_name),
			items_concurrency: config_ov.items_concurrency.or(self.items_concurrency),
		})
	}
}
