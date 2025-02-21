use crate::run::{Runtime, RuntimeContext};
use crate::script::process_lua_eval_result;
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
	let res = lua.load(code).eval::<mlua::Value>();
	let res_lua_value = process_lua_eval_result(lua, res, code)?;
	let value = serde_json::to_value(&res_lua_value)?;
	Ok(value)
}
