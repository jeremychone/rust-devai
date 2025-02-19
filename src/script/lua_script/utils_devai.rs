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
use mlua::{Lua, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<()> {
	let table = lua.create_table()?;

	let before_all_response_fn = lua.create_function(devai_before_all_response)?;
	table.set("before_all_response", before_all_response_fn)?;

	let skip_fn = lua.create_function(devai_skip)?;
	table.set("skip", skip_fn)?;

	let globals = lua.globals();
	globals.set("devai", table)?;

	Ok(())
}

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
