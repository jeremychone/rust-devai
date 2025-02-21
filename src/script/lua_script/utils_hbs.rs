//! Defines the `hbs` module for Lua Handlebars integration.
//!
//! ## Lua Documentation
//! The `hbs` module exposes functions that render Handlebars templates with
//! provided data. This is useful for dynamically generating content within Lua scripts.
//!
//! ### Functions
//! * `hbs.render(hbs_tmpl: string, data: table) -> string`

use crate::run::RuntimeContext;
use crate::support::hbs::hbs_render;
use crate::Result;
use handlebars::JsonValue;
use mlua::{Lua, Table, Value};
use std::collections::HashMap;

/// Initializes the Handlebars module for Lua.
///
/// This function creates a Lua table with the available Handlebars functions.
/// Register this table under a namespace (for example, `utils.hbs`) to make the
/// functions available in your Lua scripts.
pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> mlua::Result<Table> {
	let table = lua.create_table()?;
	let render_fn = lua.create_function(lua_hbs_render)?;
	table.set("render", render_fn)?;
	Ok(table)
}

/// Renders a Handlebars template using provided data.
///
/// ### Lua Documentation
/// ```lua
/// -- Render a template using a data table
/// local tmpl = "Hello, {{name}}!"
/// local data = { name = "Alice" }
/// local result = utils.hbs.render(tmpl, data)
/// print(result)  -- Output: Hello, Alice!
/// ```
///
/// # Parameters:
/// - `hbs_tmpl` (string): The Handlebars template string.
/// - `data` (table): A table containing key-value pairs for the template.
///
/// # Returns:
/// - (string): The rendered template.

fn lua_hbs_render(lua: &mlua::Lua, (hbs_tmpl, data): (String, mlua::Table)) -> mlua::Result<String> {
	// Convert the Lua table to a serde_json::Value using serde_json::to_value.
	let data_json: serde_json::Value = serde_json::to_value(data)
		.map_err(|e| mlua::Error::external(format!("Failed to convert Lua table to JSON: {}", e)))?;

	// Convert the JSON object into a HashMap.
	let mut data_map: std::collections::HashMap<String, serde_json::Value> = match data_json {
		serde_json::Value::Object(map) => map.into_iter().collect(),
		_ => return Err(mlua::Error::external("Expected a JSON object for data")),
	};

	// Helper function to process JSON values while preserving nested objects.
	fn process_value(val: serde_json::Value) -> serde_json::Value {
		match val {
			serde_json::Value::Object(mut obj) => {
				if let Some(serde_json::Value::String(content)) = obj.get("content") {
					serde_json::Value::String(content.clone()) // Extract "content" field if present
				} else {
					serde_json::Value::Object(
						obj.into_iter()
							.map(|(k, v)| (k, process_value(v))) // Recursively process nested objects
							.collect(),
					)
				}
			}
			serde_json::Value::Array(arr) => serde_json::Value::Array(arr.into_iter().map(process_value).collect()),
			serde_json::Value::String(_) => val,             // Keep strings unchanged
			_ => serde_json::Value::String(val.to_string()), // Convert other types to strings
		}
	}

	// Iterate over the data_map and process each value.
	for (_k, v) in data_map.iter_mut() {
		*v = process_value(v.take());
	}

	// Render the Handlebars template using the updated data_map.
	let rendered = hbs_render(&hbs_tmpl, &data_map)
		.map_err(|e| mlua::Error::external(format!("Handlebars render error: {}", e)))?;
	Ok(rendered)
}

// region: --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::run::Runtime;
	use mlua::Lua;

	#[tokio::test]
	async fn test_lua_hbs_render() -> Result<()> {
		// Setup a test runtime and Lua engine.
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let lua_engine = runtime.new_lua_engine()?;
		let globals = lua_engine.globals();

		// Initialize the hbs module and register it under the globals.
		let hbs_module = init_module(&lua_engine)?;
		globals.set("hbs", hbs_module)?;

		// Lua script to render a Handlebars template.
		let lua_script = r#"
            local tmpl = "Hello, {{name}}!"
            local data = { name = "Alice" }
            return hbs.render(tmpl, data)
        "#;

		let result: String = lua_engine.load(lua_script).eval()?;
		assert_eq!(result, "Hello, Alice!");

		Ok(())
	}
}

// endregion: --- Tests
