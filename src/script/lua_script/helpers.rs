use mlua::{Table, Value};
use std::sync::Arc;

// Return a Vec<String> from a lua Value which can be String or Array of strings
pub fn to_vec_of_strings(value: Value, err_prefix: &'static str) -> mlua::Result<Vec<String>> {
	match value {
		// If the value is a string, return a Vec with that single string.
		Value::String(lua_string) => {
			let string_value = lua_string.to_str()?.to_string();
			Ok(vec![string_value])
		}

		// If the value is a table, try to interpret it as a list of strings.
		Value::Table(lua_table) => {
			let mut result = Vec::new();

			// Iterate over the table to collect strings.
			for pair in lua_table.sequence_values::<String>() {
				match pair {
					Ok(s) => result.push(s),
					Err(_) => {
						return Err(mlua::Error::FromLuaConversionError {
							from: "table",
							to: "Vec<String>".to_string(),
							message: Some(format!("{err_prefix} - Table contains non-string values")),
						})
					}
				}
			}

			Ok(result)
		}

		// Otherwise, return an error because the value is neither a string nor a list.
		_ => Err(mlua::Error::FromLuaConversionError {
			from: "unknown",
			to: "Vec<String>".to_string(),
			message: Some(format!("{err_prefix} - Expected a string or a list of strings")),
		}),
	}
}

// region:    --- LuaTableError

#[derive(Debug)]
struct LuaTableError {
	table: Table,
}

impl std::error::Error for LuaTableError {}

impl std::fmt::Display for LuaTableError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// Convert the Lua table to a string representation for the error message
		let mut output = String::new();
		for pair in self.table.pairs::<Value, Value>() {
			match pair {
				Ok((key, value)) => {
					output.push_str(&format!("{:?}: {:?}, ", key, value));
				}
				Err(_) => {
					output.push_str("Error reading table values, ");
				}
			}
		}
		write!(f, "LuaTableError: {{ {} }}", output.trim_end_matches(", "))
	}
}

pub fn make_table_external_error(table: Table) -> mlua::Error {
	mlua::Error::ExternalError(Arc::new(LuaTableError { table }))
}

// endregion: --- LuaTableError
