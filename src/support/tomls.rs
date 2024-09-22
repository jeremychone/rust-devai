//! Crate utility for toml
//!
//! Note: The goal is that all get serialized to serded_json as this is the cannonical format for now.

use crate::Result;

use serde_json::Value as JsonValue;
use toml::Value as TomlValue;

pub fn parse_toml(toml_content: &str) -> Result<JsonValue> {
	// Parse the TOML string into a TOML value
	let toml_value: TomlValue = toml::from_str(toml_content)?;

	// Convert the TOML value to a serde_json::Value
	let json_value = serde_json::to_value(toml_value)?;

	Ok(json_value)
}
