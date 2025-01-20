//! Defines the `json` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `json` module exposes functions to parse and stringify JSON content.
//!
//! ### Functions
//! * `utils.json.parse(content: string) -> table`
//! * `utils.json.stringify(content: table) -> string`
//! * `utils.json.stringify_to_line(content: table) -> string`

use crate::run::RuntimeContext;
use crate::{Error, Result};
use mlua::{Lua, LuaSerdeExt, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_fn = lua.create_function(move |lua, content: String| parse(lua, content))?;
	let stringify_fn = lua.create_function(move |lua, content: Value| stringify(lua, content))?;
	let stringify_to_line_fn = lua.create_function(move |lua, content: Value| stringify_to_line(lua, content))?;

	table.set("parse", parse_fn)?;
	table.set("stringify", stringify_fn)?;
	table.set("stringify_to_line", stringify_to_line_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Parse a JSON string into a table.
///
/// ```lua
/// -- API Signature
/// utils.json.parse(content: string) -> table
/// ```
///
/// Parse a JSON string into a table that can be used in the Lua script.
///
/// ### Example
/// ```lua
/// local json_str = '{"name": "John", "age": 30}'
/// local obj = utils.json.parse(json_str)
/// print(obj.name) -- prints "John"
/// ```
///
/// ### Returns
///
/// Returns a table representing the parsed JSON.
///
/// ### Exception
///
/// ```lua
/// {
///   error = string  -- Error message from JSON parsing
/// }
/// ```
fn parse(lua: &Lua, content: String) -> mlua::Result<Value> {
	match serde_json::from_str::<serde_json::Value>(&content) {
		Ok(val) => Ok(lua.to_value(&val)?),
		Err(err) => Err(Error::cc("utils.json.parse failed", err).into()),
	}
}

/// ## Lua Documentation
///
/// Stringify a table into a JSON string with pretty formatting.
///
/// ```lua
/// -- API Signature  
/// utils.json.stringify(content: table) -> string
/// ```
///
/// Convert a table into a JSON string with pretty formatting using tab indentation.
///
/// ### Example
/// ```lua
/// local obj = {
///     name = "John",
///     age = 30
/// }
/// local json_str = utils.json.stringify(obj)
/// -- Result will be:
/// -- {
/// --     "name": "John",
/// --     "age": 30
/// -- }
/// ```
///
/// ### Returns
///
/// Returns a formatted JSON string.
///
/// ### Exception
///
/// ```lua
/// {
///   error = string  -- Error message from JSON stringification
/// }
/// ```
fn stringify(_lua: &Lua, content: Value) -> mlua::Result<String> {
	match serde_json::to_value(content) {
		Ok(val) => match serde_json::to_string_pretty(&val) {
			Ok(str) => Ok(str),
			Err(err) => Err(Error::custom(format!("Fail to stringify. {}", err)).into()),
		},
		Err(err) => Err(Error::custom(format!("Fail to convert value. {}", err)).into()),
	}
}

/// ## Lua Documentation
///
/// Stringify a table into a single line JSON string.
///
/// Good for newline json
///
/// ```lua
/// -- API Signature
/// utils.json.stringify_to_line(content: table) -> string
/// ```
///
/// Convert a table into a single line JSON string.
///
/// ### Example
/// ```lua
/// local obj = {
///     name = "John",
///     age = 30
/// }
/// local json_str = utils.json.stringify_to_line(obj)
/// -- Result will be:
/// -- {"name":"John","age":30}
/// ```
///
/// ### Returns
///
/// Returns a single line JSON string.
///
/// ### Exception
///
/// ```lua
/// {
///   error = string  -- Error message from JSON stringification
/// }
/// ```
fn stringify_to_line(_lua: &Lua, content: Value) -> mlua::Result<String> {
	match serde_json::to_value(content) {
		Ok(val) => match serde_json::to_string(&val) {
			Ok(str) => Ok(str),
			Err(err) => Err(Error::custom(format!("utils.json.stringify fail to stringify. {}", err)).into()),
		},
		Err(err) => Err(Error::custom(format!("utils.json.stringify fail to convert value. {}", err)).into()),
	}
}

#[cfg(test)]
mod tests {
	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_lua_json_parse() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            local content = '{"name": "John", "age": 30}'
            return utils.json.parse(content)
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert_eq!(res.x_get_str("name")?, "John");
		assert_eq!(res.x_get_i64("age")?, 30);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_json_parse_invalid() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            local ok, err = pcall(function()
                local content = "{invalid_json}"
                return utils.json.parse(content)
            end)
            if not ok then
                return err
            end
            return "should not reach here"
        "#;

		let Err(err) = run_reflective_agent(script, None).await else {
			return Err("Should have been an Error".into());
		};

		let err = format!("{err}");
		assert!(
			err.contains("json.parse failed"),
			"err should contain json.parse failed"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_json_stringify() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            local obj = {
                name = "John",
                age = 30
            }
            return utils.json.stringify(obj)
        "#;

		let res = run_reflective_agent(script, None).await?;
		let result = res.as_str().unwrap();

		let parsed: serde_json::Value = serde_json::from_str(result)?;
		assert_eq!(parsed["name"], "John");
		assert_eq!(parsed["age"], 30);
		assert!(result.contains("\n"));
		assert!(result.contains("  "));

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_json_stringify_complex() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            local obj = {
                name = "John",
                age = 30,
                address = {
                    street = "123 Main St",
                    city = "New York"
                },
                hobbies = {"reading", "gaming"}
            }
            return utils.json.stringify(obj)
        "#;

		let res = run_reflective_agent(script, None).await?;
		let result = res.as_str().ok_or("should be string")?;

		let parsed: serde_json::Value = serde_json::from_str(result)?;
		assert_eq!(parsed["name"], "John");
		assert_eq!(parsed["age"], 30);
		assert_eq!(parsed["address"]["street"], "123 Main St");
		assert_eq!(parsed["hobbies"][0], "reading");
		assert!(result.contains("\n"));
		assert!(result.contains("  "));

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_json_stringify_to_line() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            local obj = {
                name = "John",
                age = 30,
                address = {
                    street = "123 Main St",
                    city = "New York"
                },
                hobbies = {"reading", "gaming"}
            }
            return utils.json.stringify_to_line(obj)
        "#;

		let res = run_reflective_agent(script, None).await?;
		let res = res.as_str().ok_or("should be string")?;

		assert_contains(res, r#""name":"John""#);
		assert_not_contains(res, "\n");
		assert_not_contains(res, "  ");

		Ok(())
	}
}
