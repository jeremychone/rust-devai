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

	temperature: Option<f64>,

	// Runtime settings
	inputs_concurrency: Option<usize>,
}

// Getters
impl AgentConfig {
	pub fn model(&self) -> Option<&str> {
		self.model.as_deref()
	}

	pub fn inputs_concurrency(&self) -> Option<usize> {
		self.inputs_concurrency
	}

	pub fn temperature(&self) -> Option<f64> {
		self.temperature
	}
}

// Constructors
impl AgentConfig {
	/// Creates a new `AgentConfig` with the specified model name.
	#[allow(unused)]
	pub fn new(model_name: impl Into<String>) -> Self {
		AgentConfig {
			model: Some(model_name.into()),
			temperature: None,
			inputs_concurrency: None,
		}
	}

	/// Creates a new `AgentConfig` from a Value document (either from `cargo.toml` or `# Config` section).
	pub fn from_value(value: Value) -> Result<AgentConfig> {
		let model = value.x_get("/genai/model").ok();
		let temperature: Option<f64> = value.x_get("/genai/temperature").ok();

		let inputs_concurrency = value.x_get("/runtime/inputs_concurrency").ok();

		Ok(AgentConfig {
			model,
			temperature,
			inputs_concurrency,
		})
	}

	/// Merges the current config with a new config value, returning the merged `AgentConfig`.
	pub fn merge(self, value: Value) -> Result<AgentConfig> {
		let config_ov = AgentConfig::from_value(value)?;

		Ok(AgentConfig {
			model: config_ov.model.or(self.model),
			temperature: config_ov.temperature.or(self.temperature),
			inputs_concurrency: config_ov.inputs_concurrency.or(self.inputs_concurrency),
		})
	}
}
