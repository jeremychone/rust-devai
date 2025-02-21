//! Defines the `devai` module for Lua.
//!
//! ---
//!
//! ## Lua Documentation
//! The `utils.devai` module exposes functions for generating structured responses for the devai runtime.
//!
//! ### Functions
//! * `utils.devai.before_all_response(data: any) -> table`
//! * `utils.devai.skip(reason?: string) -> table`

use crate::run::RuntimeContext;
use crate::Result;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let before_all_response_fn = lua.create_function(devai_before_all_response)?;
	table.set("before_all_response", before_all_response_fn)?;

	let skip_fn = lua.create_function(devai_skip)?;
	table.set("skip", skip_fn)?;

	let globals = lua.globals();
	globals.set("devai", &table)?;

	// NOTE: For this one, we do not really need for now to return,
	//       but to have consistent with setup_lua

	Ok(table)
}

// region: --- Lua Functions

/// ## Lua Documentation
///
/// Returns a response that overrides inputs.
///
/// ```lua
/// -- API Signature
/// utils.devai.before_all_response(data: any) -> table
/// ```
///
/// Returns a table with the following structure:
/// ```lua
/// {
///   _devai_ = {
///     kind = "BeforeAllResponse",
///     data = <data passed to function>
///   }
/// }
/// ```
fn devai_before_all_response(lua: &Lua, data: Value) -> mlua::Result<Value> {
	let inner = lua.create_table()?;
	inner.set("kind", "BeforeAllResponse")?;
	inner.set("data", data)?;
	let outer = lua.create_table()?;
	outer.set("_devai_", inner)?;

	Ok(Value::Table(outer))
}

/// ## Lua Documentation
///
/// Returns a response indicating a skip action for the input cycle.
///
/// ```lua
/// -- API Signature
/// utils.devai.skip(reason?: string) -> table
/// ```
///
/// Returns a table with the following structure:
/// ```lua
/// {
///   _devai_ = {
///     kind = "Skip",
///     data = { reason = <reason passed to function, can be nil> }
///   }
/// }
/// ```
fn devai_skip(lua: &Lua, reason: Option<String>) -> mlua::Result<Value> {
	let data = lua.create_table()?;
	data.set("reason", reason)?;

	let inner = lua.create_table()?;
	inner.set("kind", "Skip")?;
	inner.set("data", data)?;

	let outer = lua.create_table()?;
	outer.set("_devai_", inner)?;

	Ok(Value::Table(outer))
}

// endregion: --- Lua Functions

// region:    --- Section

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};
	use serde_json::Value;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_devai_before_all_response_simple() -> Result<()> {
		// -- Setup
		let lua = setup_lua(super::init_module, "devai")?;
		let script = r#"
			return devai.before_all_response(123)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_devai_/kind")?;
		assert_eq!(kind, "BeforeAllResponse");

		let data = res.x_get_i64("/_devai_/data")?;
		assert_eq!(data, 123);
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_devai_skip_with_reason() -> Result<()> {
		// -- Setup
		let lua = setup_lua(super::init_module, "devai")?;
		let script = r#"
			return devai.skip("Not applicable")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_devai_/kind")?;
		assert_eq!(kind, "Skip");

		let reason = res.x_get_str("/_devai_/data/reason")?;
		assert_eq!(reason, "Not applicable");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_devai_skip_without_reason() -> Result<()> {
		// -- Setup
		let lua = setup_lua(super::init_module, "devai")?;
		let script = r#"
			return devai.skip()
		"#;

		// -- Exec
		let mut res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_devai_/kind")?;
		assert_eq!(kind, "Skip");

		// NOTE: For now, even if we ask Option<Value>, on /_devai_/data/reason, we get an error. Should probably be fix in value-ext
		let data = res.x_remove::<Value>("/_devai_/data")?;
		let reason = data.x_get::<String>("reason").ok();
		assert!(reason.is_none(), "reason should be none");
		Ok(())
	}
}

// endregion: --- Section
