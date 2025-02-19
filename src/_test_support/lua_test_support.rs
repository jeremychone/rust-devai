use crate::run::{Runtime, RuntimeContext};
use crate::Result;
use mlua::{Lua, Table};
use serde_json::Value;

/// Sets up a Lua instance with both functions registered under `utils.` utils_name.
pub fn setup_lua<F>(init_fn: F, utils_name: &str) -> Result<Lua>
where
	F: FnOnce(&Lua, &RuntimeContext) -> Result<Table>,
{
	let runtime = Runtime::new_test_runtime_sandbox_01()?;

	let lua = Lua::new();
	let globals = lua.globals();
	let utils = lua.create_table().unwrap();

	let path_table = init_fn(&lua, &runtime.context())?;
	utils.set(utils_name, path_table).unwrap();
	globals.set("utils", utils).unwrap();

	Ok(lua)
}

pub fn eval_lua(lua: &Lua, code: &str) -> Result<Value> {
	let lua_value = lua.load(code).eval::<mlua::Value>()?;
	let value = serde_json::to_value(&lua_value)?;
	Ok(value)
}
