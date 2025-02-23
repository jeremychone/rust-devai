use crate::Result;
use crate::hub::get_hub;
use genai::chat::ChatOptions;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use value_ext::JsonValueExt;

/// Configuration for the Agent, defined in `.aipack/config.toml` and
/// optionally overridden in the `# Config` section of the Command Agent Markdown.
///
/// Note: The values are flattened for simplicity but may be nested in the future.
#[derive(Debug, Clone, Deserialize, Default, Serialize)]
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

// region:    --- Froms

impl From<&AgentOptions> for ChatOptions {
	fn from(agent_options: &AgentOptions) -> Self {
		let mut chat_options = ChatOptions::default();
		if let Some(temp) = agent_options.temperature() {
			chat_options.temperature = Some(temp);
		}
		chat_options
	}
}

// endregion: --- Froms

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

	pub fn merge_new(&self, aliases_ov: Option<ModelAliases>) -> ModelAliases {
		let mut inner: HashMap<String, String> = self.inner.clone();
		if let Some(aliases) = aliases_ov {
			for (k, v) in aliases.inner {
				inner.insert(k, v);
			}
		}
		ModelAliases { inner }
	}
}

impl mlua::FromLua for ModelAliases {
	fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
		match value {
			mlua::Value::Table(aliases_table) => {
				let mut aliases = HashMap::new();
				for pair in aliases_table.pairs::<String, String>() {
					let (k, v) = pair.map_err(|err| {
						mlua::Error::runtime(format!(
							"model_aliases value type is invalid. Should be string.\n    Cause: {err}"
						))
					})?; // TODO: need to return informative error
					aliases.insert(k, v);
				}
				Ok(ModelAliases { inner: aliases })
			}
			other => Err(mlua::Error::runtime(format!(
				r#"model_aliases invalid.\n    Cause: for agent options must be of type table (e.g., {{ small = "gpt-4o-mini" }}), but was {other:?}"#
			))),
		}
	}
}

impl mlua::IntoLua for &ModelAliases {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		for (k, v) in self.inner.iter() {
			table.set(k.as_str(), v.as_str())?;
		}
		Ok(mlua::Value::Table(table))
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
	///
	/// Note: This will consume both, avoiding any new allocation
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

	pub fn merge_new(&self, options_ov: AgentOptions) -> Result<AgentOptions> {
		let model_aliases = match &self.model_aliases {
			Some(aliases) => Some(aliases.merge_new(options_ov.model_aliases)),
			None => options_ov.model_aliases.clone(),
		};

		Ok(AgentOptions {
			legacy: options_ov.legacy, // only take the value of the legacy
			model: options_ov.model.or(self.model.clone()),
			temperature: options_ov.temperature.or(self.temperature),
			input_concurrency: options_ov.input_concurrency.or(self.input_concurrency),
			model_aliases,
		})
	}
}

// region:    --- IntoLua

impl mlua::IntoLua for &AgentOptions {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("model", self.model())?;
		table.set("resolved_model", self.resolve_model())?;
		table.set("temperature", self.temperature)?;
		table.set("input_concurrency", self.input_concurrency)?;

		let model_aliases = self.model_aliases.as_ref();
		table.set("model_aliases", model_aliases)?;

		Ok(mlua::Value::Table(table))
	}
}

impl mlua::FromLua for AgentOptions {
	fn from_lua(value: mlua::Value, lua: &mlua::Lua) -> mlua::Result<Self> {
		if let mlua::Value::Table(table) = value {
			let model = table.get::<Option<String>>("model")?;
			let temperature = table.get::<Option<f64>>("temperature")?;
			let input_concurrency = table.get::<Option<usize>>("input_concurrency")?;

			// --
			let model_aliases = table.get::<Option<mlua::Value>>("model_aliases")?;
			let model_aliases = model_aliases.map(|v| ModelAliases::from_lua(v, lua)).transpose()?;

			let options = AgentOptions {
				legacy: false,
				model,
				temperature,
				input_concurrency,
				model_aliases,
			};

			Ok(options)
		} else {
			Err(mlua::Error::runtime("Agent Options must be a table"))
		}
	}
}
// endregion: --- IntoLua

// region:    --- Parsing

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
  - Run 'aip init' (this will create a new config.toml).
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

// endregion: --- Parsing

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
	use mlua::{FromLua, IntoLua};

	#[test]
	fn test_options_current_with_aliases() -> Result<()> {
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
	fn test_options_lua_from() -> Result<()> {
		// -- Setup & Fixtures
		let lua = mlua::Lua::new();
		let options_chunk = lua.load(
			r#"
return {
	model = "gpt-4o-mini",
	temperature = 0.3,
	model_aliases = { small = "flash-001" },
	item_concurrency = nil, -- same as absent
}"#,
		);
		let options_lua = options_chunk.eval::<mlua::Value>()?;

		// -- Exec
		let options = AgentOptions::from_lua(options_lua, &lua)?;

		// -- Check
		assert_eq!(options.model(), Some("gpt-4o-mini"));
		assert_eq!(options.temperature(), Some(0.3));
		assert!(
			options.input_concurrency().is_none(),
			"input concurrency should be none"
		);
		assert_eq!(options.get_model_for_alias("small"), Some("flash-001"));
		assert!(
			options.get_model_for_alias("non-existent").is_none(),
			"Model alias 'non-existent' should be none"
		);

		Ok(())
	}

	#[test]
	fn test_options_lua_into() -> Result<()> {
		// -- Setup & Fixtures
		let lua = mlua::Lua::new();
		let options = parse_toml(
			r#"
	model = "gpt-4o-mini"
	temperature = 0.3
	model_aliases = { small = "flash-001" }		
		"#,
		)?;
		let options = AgentOptions::from_options_value(options.clone())?;

		// -- Exec
		let options_lua = options.into_lua(&lua)?;

		// -- Check
		let options_table = options_lua.as_table().ok_or("Should be a table")?;
		assert_eq!(&options_table.get::<String>("model")?, "gpt-4o-mini");
		assert_eq!(options_table.get::<f64>("temperature")?, 0.3);
		let aliases_table = options_table.get::<mlua::Value>("model_aliases")?;
		let aliases_table = aliases_table.as_table().ok_or("model_aliases should be table")?;
		assert_eq!(&aliases_table.get::<String>("small")?, "flash-001");

		Ok(())
	}
}

// endregion: --- Tests
