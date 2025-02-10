use crate::run::RuntimeContext;
use mlua::Lua;
use mlua::Table;
use simple_fs::{list_files, read_to_string, SFile, SPath};
use crate::support::tomls::parse_toml;

/// Convert JSON to Lua Table
fn json_to_lua_table(lua: &Lua, json_value: &serde_json::Value) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    match json_value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                match value {
                    serde_json::Value::String(s) => table.set(key.as_str(), s.as_str())?,
                    serde_json::Value::Number(n) => table.set(key.as_str(), n.as_f64().unwrap_or(0.0))?,
                    serde_json::Value::Bool(b) => table.set(key.as_str(), *b)?,
                    serde_json::Value::Array(arr) => {
                        let arr_table = lua.create_table()?;
                        for (i, item) in arr.iter().enumerate() {
                            match item {
                                serde_json::Value::String(s) => arr_table.set(i + 1, s.as_str())?,
                                serde_json::Value::Number(n) => arr_table.set(i + 1, n.as_f64().unwrap_or(0.0))?,
                                serde_json::Value::Bool(b) => arr_table.set(i + 1, *b)?,
                                _ => {}
                            }
                        }
                        table.set(key.as_str(), arr_table)?;
                    }
                    serde_json::Value::Object(_) => {
                        let sub_table = json_to_lua_table(lua, value)?;
                        table.set(key.as_str(), sub_table)?;
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    Ok(table)
}

pub fn init_module(lua: &Lua, runtime_context: &RuntimeContext) -> mlua::Result<()> {
 	let config_path = runtime_context.dir_context().devai_dir().get_config_toml_path()?;
	let config_content = read_to_string(config_path).unwrap_or_default();
	let config_value = parse_toml(&config_content).unwrap_or_default();
    let lua_table = json_to_lua_table(lua, &config_value)?;
    lua.globals().set("devai_config", lua_table)?;

    Ok(())
}
