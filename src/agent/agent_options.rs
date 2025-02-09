use crate::hub::get_hub;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use value_ext::JsonValueExt;

/// Configuration for the Agent, defined in `.devai/config.toml` and
/// optionally overridden in the `# Config` section of the Command Agent Markdown.
///
/// Note: The values are flattened for simplicity but may be nested in the future.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentOptions {
	#[serde(default)]
	legacy: bool,

	// The raw model name of the configuration
	model: Option<String>,

	temperature: Option<f64>,

	// Runtime settings
	input_concurrency: Option<usize>,

	model_aliases: Option<ModelAliases>,
}

// region:    --- ModelAliases

/// TODO Must have a Arc<inner> for perf
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelAliases {
	/// The `{name: model_name}` hashmap
	#[serde(flatten)]
	inner: HashMap<String, String>,
}

impl ModelAliases {
	pub fn merge(mut self, aliases_ov: Option<ModelAliases>) -> ModelAliases {
		if let Some(aliases) = aliases_ov {
			for (k, v) in aliases.inner {
				self.inner.insert(k, v);
			}
		}
		self
	}
}

// endregion: --- ModelAliases

// Getters
impl AgentOptions {
	/// Returns the raw model name from this options given in the config/options
	/// (This name is not resolved with the model aliases)
	pub fn model(&self) -> Option<&str> {
		self.model.as_deref()
	}

	/// Returns the resolved model
	pub fn resolve_model(&self) -> Option<&str> {
		let model = self.model.as_deref()?;

		match self.model_aliases {
			Some(_) => self.get_model_for_alias(model).or(Some(model)),
			None => Some(model),
		}
	}

	pub fn input_concurrency(&self) -> Option<usize> {
		self.input_concurrency
	}

	pub fn temperature(&self) -> Option<f64> {
		self.temperature
	}

	#[allow(unused)]
	fn get_model_for_alias(&self, alias: &str) -> Option<&str> {
		self.model_aliases
			.as_ref()
			.and_then(|aliases| aliases.inner.get(alias).map(|s| s.as_str()))
	}
}

// Constructors
impl AgentOptions {
	/// Creates a new `AgentOptions` from a Value document (either from `cargo.toml` or `# Config` section).
	/// Note: It will try to first parse it with the new format (default_options), and then, with the legacy format (genai/runtime)
	///
	/// TODO: Needs to have another function, from_options_value for when in the new `# Options` section which will follow the section format)
	pub fn from_config_value(value: Value) -> Result<AgentOptions> {
		let options = match Self::from_current_config(value)? {
			OptionsParsing::Parsed(agent_options) => agent_options,
			OptionsParsing::Unparsed(value) => Self::from_legacy_0_5_9_config(value)?,
		};

		Ok(options)
	}

	/// Creates a new `AgentOptions` from the flatten `options` structure.
	/// This is mostly for when the agent file as a `# Options` sections (which replaces the `# Config`)
	pub fn from_options_value(value: Value) -> Result<AgentOptions> {
		let options = serde_json::from_value(value)?;

		Ok(options)
	}

	/// Merge the current options with a new options value, returning the merged `AgentOptions`.
	pub fn merge(self, options_ov: AgentOptions) -> Result<AgentOptions> {
		let model_aliases = match self.model_aliases {
			Some(aliases) => Some(aliases.merge(options_ov.model_aliases)),
			None => options_ov.model_aliases,
		};

		Ok(AgentOptions {
			legacy: options_ov.legacy, // only take the value of the legacy
			model: options_ov.model.or(self.model),
			temperature: options_ov.temperature.or(self.temperature),
			input_concurrency: options_ov.input_concurrency.or(self.input_concurrency),
			model_aliases,
		})
	}
}

enum OptionsParsing {
	Parsed(AgentOptions),
	Unparsed(Value),
}

/// private parsers
impl AgentOptions {
	/// Parse the config toml json value with legacy support.
	///
	/// - Returns None if it is not a latest config format (with `default_options`)
	/// - Returns the AgentOptions is valid config toml
	/// - Returns error if something wrong in the format
	fn from_current_config(config_value: Value) -> Result<OptionsParsing> {
		// first, check if it has some invalid value
		if config_value.pointer("/default-options").is_some() {
			return Err("Config [default-options] is invalid. Use [default_options] (with _ and not -)".into());
		}

		let Some(config_value) = config_value.x_get::<Value>("/default_options").ok() else {
			return Ok(OptionsParsing::Unparsed(config_value));
		};

		let options = Self::from_options_value(config_value)?;

		Ok(OptionsParsing::Parsed(options))
	}

	/// Parse the legacy 0.5.9 config format, with `genai.` and `runtime.`
	fn from_legacy_0_5_9_config(config_value: Value) -> Result<AgentOptions> {
		let model = config_value.x_get("/genai/model").ok();
		let temperature: Option<f64> = config_value.x_get("/genai/temperature").ok();

		let input_concurrency = config_value.x_get("/runtime/input_concurrency").ok();

		// -- send a warning message
		let hub = get_hub();
		hub.publish_sync(
			"\
==== WARNING

The config.toml format has been updated. 
Your current config.toml uses the legacy format. It still works, but it is recommended to update it. 

To update your config.toml:
  - Rename your current config.toml to config-old.toml (or any name you prefer).
  - Run 'devai init' (this will create a new config.toml).
  - Manually transfer the values from your config-old.toml to the new config.toml.

====
",
		);

		Ok(AgentOptions {
			legacy: true,
			model,
			temperature,
			input_concurrency,
			model_aliases: None,
		})
	}
}

/// Implementations for various test.
#[cfg(test)]
impl AgentOptions {
	/// Creates a new `AgentOptions` with the specified model name. (for test)
	pub fn new(model_name: impl Into<String>) -> Self {
		AgentOptions {
			legacy: false,
			model: Some(model_name.into()),
			temperature: None,
			input_concurrency: None,
			model_aliases: None,
		}
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::support::tomls::parse_toml;

	#[test]
	fn test_config_current_with_aliases() -> Result<()> {
		// -- Setup & Fixtures
		let config_content = simple_fs::read_to_string("./tests-data/config/config-current-with-aliases.toml")?;
		let config_value = serde_json::to_value(&parse_toml(&config_content)?)?;

		// -- Exec
		let options = AgentOptions::from_config_value(config_value)?;

		// -- Check
		assert!(!options.legacy, "Should NOT be legacy");
		assert_eq!(options.model(), Some("gpt-4o-mini"));
		assert_eq!(options.temperature(), Some(0.0));
		assert_eq!(options.input_concurrency(), Some(6));
		assert_eq!(
			options.get_model_for_alias("small").ok_or("Should have an alias for small")?,
			"gemini-2.0-flash-001"
		);

		Ok(())
	}

	#[test]
	fn test_config_legacy_0_5_9() -> Result<()> {
		// -- Setup & Fixtures
		let config_content = simple_fs::read_to_string("./tests-data/config/config-v_0_5_09.toml")?;
		let config_value = serde_json::to_value(&parse_toml(&config_content)?)?;

		// -- Exec
		let options = AgentOptions::from_config_value(config_value)?;

		// -- Check
		assert!(options.legacy, "Should be legacy");
		assert_eq!(options.model(), Some("gpt-4o-mini"));
		assert_eq!(options.temperature(), Some(0.0));
		assert_eq!(options.input_concurrency(), Some(6));
		assert!(
			options.get_model_for_alias("small").is_none(),
			" should not have any alias"
		);

		Ok(())
	}
}

// endregion: --- Tests
