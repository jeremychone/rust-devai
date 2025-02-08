use crate::run::RuntimeContext;
use mlua::{Lua, Result, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let dump_lua = lua.create_function(dump)?;
	table.set("dump", dump_lua)?;

	Ok(table)
}

// region:    --- Rust Lua Support

pub fn dump(lua: &Lua, value: Value) -> Result<String> {
	fn dump_value(_lua: &Lua, value: Value, indent: usize) -> Result<String> {
		let indent_str = "  ".repeat(indent);
		match value {
			Value::Nil => Ok("nil".to_string()),
			Value::Boolean(b) => Ok(b.to_string()),
			Value::Integer(i) => Ok(i.to_string()),
			Value::Number(n) => Ok(n.to_string()),
			Value::String(s) => {
				let s: String = s.to_str()?.to_string();
				Ok(format!("\"{}\"", s))
			}
			Value::Table(t) => {
				let mut entries: Vec<String> = Vec::new();
				for pair in t.clone().pairs::<Value, Value>() {
					let (key, val) = pair?;
					let dumped_key = match key {
						Value::String(s) => s.to_str()?.to_string(),
						_ => dump_value(_lua, key, 0)?,
					};
					let dumped_val = dump_value(_lua, val, indent + 1)?;
					entries.push(format!("{}{} = {}", "  ".repeat(indent + 1), dumped_key, dumped_val));
				}
				let inner = entries.join(",\n");
				Ok(format!("{{\n{}\n{}}}", inner, indent_str))
			}
			Value::Function(f) => {
				let name = f.info().name.unwrap_or("<anonymous>".to_string());
				Ok(format!("<function {}>", name))
			}
			Value::UserData(_) => Ok("<UserData>".to_string()),
			Value::LightUserData(_) => Ok("<LightUserData>".to_string()),
			Value::Thread(_) => Ok("<Thread>".to_string()),
			_ => Ok("<OtherType>".to_string()),
		}
	}

	dump_value(lua, value, 0)
}
// endregion: --- Rust Lua Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::assert_contains;
	use mlua::{Lua, Value};

	#[test]
	fn test_scripts_lua_dump() -> Result<()> {
		// -- Setup & Fixtures
		let lua = Lua::new();

		// Create nested tables
		let outer_table = lua.create_table().unwrap();
		let inner_table = lua.create_table().unwrap();

		// Populate inner table
		inner_table.set("key1", "value1").unwrap();
		inner_table.set("key2", 42).unwrap();

		// Add inner table and other values to outer table
		outer_table.set("nested", inner_table).unwrap();
		outer_table.set("bool", true).unwrap();
		outer_table.set("num", 3.21).unwrap();

		// -- Check
		let result = dump(&lua, Value::Table(outer_table)).unwrap();
		assert_contains(&result, r#"  bool = true"#);
		assert_contains(&result, r#"    key1 = "value1""#);
		assert_contains(&result, r#"    key2 = 42"#);

		Ok(())
	}
}

// endregion: --- Tests
