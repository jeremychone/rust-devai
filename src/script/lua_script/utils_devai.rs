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
/// Can be return in the `# Before All` Lua section to override the inputs.
///
/// ```lua
/// return devai.before_all_response({
///     -- (optional) Some data from before all (for later stage)
///     before_all = "Some before all data",
///     -- (optional) inputs generation
///     inputs = {"one", "two", "three", 4, "five"}
/// })
/// ```
///
/// ### Internals
///
/// This will return a structure like this one below, which will be understood by the devai runtime.
///
/// ```
/// "_devai_": {
///     "kind": "BeforeAllResponse",
///     "data": {
///         "inputs": ["one", "two", "three", 4, "five"],
///         "before_all": "Some before all data"
///     }
/// }
/// ``````
///
///
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
/// Can be returned `# Before All`, `# Data`, `# Output` Lua sections to tell the devai runtime
/// to skip this input or inputs cycle.
///
/// ```lua
/// return devai.skip("File " .. input.path .. " already contain the documentation")
/// ```
///
/// ### Internals
///
/// This will return a structure like this one below, which will be understood by the devai runtime.
///
/// ```
/// "_devai_": {
///     "kind": "Skip",
///     "data": {
///         "reason": "Some optional reason",
///     }
/// }
/// ``````
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
