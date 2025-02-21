//! Defines the `rust` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `rust` module exposes functions used to process Rust code.
//!
//! ### Functions
//! * `utils.rust.prune_to_declarations(code: string) -> string`

use crate::run::RuntimeContext;
use crate::support::code::run_prune_to_declarations;
use crate::Result;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let prune_fn = lua.create_function(prune_to_declarations)?;

	table.set("prune_to_declarations", prune_fn)?;

	Ok(table)
}

/// ## Lua Documentation
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
		Err(err) => Err(crate::Error::Lua(format!("Failed to prune Rust code: {}", err)).into()),
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{setup_lua, eval_lua, assert_contains};

	#[tokio::test]
	async fn test_lua_rust_prune_to_declarations() -> Result<()> {
		// -- Fixtures
		let lua = setup_lua(super::init_module, "rust")?;
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
		let script = format!("return utils.rust.prune_to_declarations({:?})", data_script);
		let res = eval_lua(&lua, &script)?;
		// -- Check
		let res = res.as_str().ok_or("Should be str")?;
		assert_contains(res, "use some::module; // and comment ");
		assert_contains(res, "async some_async_fn(some_arg: String) -> i32");
		assert_contains(res, "fn some_normal()");
		assert!(!res.contains(r#"let some = "code";"#), "should NOT contain let some ...");
		assert!(!res.contains("// DOING SOME STUFF"), "DOING SOME STUFF");
		Ok(())
	}
}

// endregion: --- Tests
