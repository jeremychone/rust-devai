//! Defines the `rust` module, used in the lua engine.
//!
//! ---
//!
//! ## LUA documentation
//! The `rust` module exposes functions used to process Rust code.
//!
//! ### Functions
//! * `utils.rust.prune_to_declarations(code: string) -> string`

use crate::run::RuntimeContext;
use crate::script::lua_script::helpers::make_table_external_error;
use crate::support::code::run_prune_to_declarations;
use crate::Result;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let prune_fn = lua.create_function(prune_to_declarations)?;

	table.set("prune_to_declarations", prune_fn)?;

	Ok(table)
}

/// ## LUA Documentation
/// ```lua
/// utils.rust.prune_to_declarations(code: string) -> string
/// ```
///
/// Trims Rust code to keep only function declarations by replacing function bodies with `{ ... }`.
/// Preserves comments, whitespace, and non-function code structures.
///
/// Example:
/// ```lua
/// local code = "fn add(a: i32, b: i32) -> i32 { a + b }"
/// local result = utils.rust.prune_to_declarations(code)
/// -- result will be: "fn add(a: i32, b: i32) -> i32 { ... }"
/// ```
fn prune_to_declarations(lua: &Lua, code: String) -> mlua::Result<Value> {
	match run_prune_to_declarations(&code) {
		Ok(result) => Ok(Value::String(lua.create_string(&result)?)),
		Err(err) => {
			let error_table = lua.create_table()?;
			error_table.set("error", format!("Failed to prune Rust code: {}", err))?;
			Err(make_table_external_error(error_table))
		}
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;
	use serde_json::Value;

	#[tokio::test]
	async fn test_lua_rust_prune_to_declarations() -> Result<()> {
		// -- Fixtures
		let data_script = r#"
//! Some top comment 

use some::module; // and comment 

/// Some comment
pub fn async some_async_fn(some_arg: String) -> i32{
   let some = "code";
	 123
}

// Some fn normal
fn some_normal() {
		// DOING SOME STUFF
		// some fn stuff
}	 
		"#;

		// -- Exec
		let res = run_reflective_agent(
			r#"return utils.rust.prune_to_declarations(input)"#,
			Some(Value::String(data_script.to_string())),
		)
		.await?;

		// -- Check
		let res = res.as_str().ok_or("Should be str")?;
		assert!(
			res.contains("use some::module; // and comment "),
			"should contain use ..."
		);
		assert!(
			res.contains("async some_async_fn(some_arg: String) -> i32"),
			"should contain some_async_fn"
		);
		assert!(res.contains("fn some_normal()"), "should contain some_normal");
		assert!(
			!res.contains(r#"let some = "code";"#),
			"should NOT contain let some ..."
		);
		assert!(!res.contains("// DOING SOME STUFF"), "DOING SOME STUFF");

		Ok(())
	}
}

// endregion: --- Tests
