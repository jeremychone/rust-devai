use crate::hub::{get_hub, HubEvent};
use crate::run::paths::CUSTOM_LUA_DIR;
use crate::run::{get_devai_base_dir, RuntimeContext};
use crate::script::lua_script::{
	utils_cmd, utils_devai, utils_file, utils_git, utils_html, utils_json, utils_md, utils_path, utils_rust,
	utils_text, utils_web,
};
use crate::{Error, Result};
use mlua::{Lua, LuaSerdeExt, Table, Value};

pub struct LuaEngine {
	lua: Lua,
	#[allow(unused)]
	runtime_context: RuntimeContext,
}

/// Constructors
impl LuaEngine {
	pub fn new(runtime_context: RuntimeContext) -> Result<Self> {
		let lua = Lua::new();

		// -- init utils
		init_utils(&lua, &runtime_context)?;

		// -- init devai
		utils_devai::init_module(&lua, &runtime_context)?;

		// -- Init package.path
		init_package_path(&lua, &runtime_context)?;

		// -- Init print
		init_print(&lua)?;

		// -- Build and return
		let engine = LuaEngine { lua, runtime_context };

		Ok(engine)
	}
}

/// Public Function
impl LuaEngine {
	pub fn eval(&self, script: &str, scope: Option<Table>, addl_lua_paths: Option<&[&str]>) -> Result<Value> {
		let lua = &self.lua;

		let chunck = lua.load(script);

		let chunck = if let Some(scope) = scope {
			let env = self.upgrade_scope(scope, addl_lua_paths)?;
			chunck.set_environment(env)
		} else {
			chunck
		};

		let res = chunck.eval::<Value>();
		let res = res?;
		let res = match res {
			// Need to make sure we handle when we call with pcall(...), see test_lua_json_parse_invalid
			Value::Error(err) => {
				// for now we take the last
				// TODO: We need to handle those error better.
				let mut str_buf: Vec<String> = Vec::new();
				for err_item in err.chain() {
					if let Some(crate_error) = err_item.downcast_ref::<Error>() {
						str_buf.push(crate_error.to_string())
					} else {
						str_buf.push(err_item.to_string())
					}
				}
				if str_buf.is_empty() {
					str_buf.push(err.to_string());
				}
				return Err(Error::cc("Error when lua eval\n{}", str_buf.join("\n")));
			}
			res => res,
		};

		Ok(res)
	}

	pub fn create_table(&self) -> Result<Table> {
		let res = self.lua.create_table()?;
		Ok(res)
	}

	/// Convert a json value to a lua value.
	///
	/// IMPORTANT: Use this to covert JSON Value to Lua Value, as the default mlua to_value,
	///            converts serde_json::Value::Null to Lua user data, and not mlua::Value::Nil,
	///            and we want it for devai.
	pub fn serde_to_lua_value(&self, val: serde_json::Value) -> Result<Value> {
		let res = match val {
			serde_json::Value::Null => Value::Nil,
			other => self.lua.to_value(&other)?,
		};
		Ok(res)
	}

	/// Upgrade a custom scope to full scope with all of the globals added.
	/// NOTE: A `base_lua_path` is the container of the `lua/` dir. So
	///       `base_lua_path = /some/dir`, the path added to lua package path will be `/some/dir/lua/?.lua,/some/dir/lua/?/init.lua`
	fn upgrade_scope(&self, scope: Table, addl_base_lua_paths: Option<&[&str]>) -> Result<Table> {
		// Get the globals table
		let globals = self.lua.globals();

		// Iterate over globals and add them to the scope table
		for pair in globals.pairs::<Value, Value>() {
			let (key, value) = pair?;
			scope.set(key, value)?; // Add each global to the scope table
		}

		// -- Prepend the additional lua path
		if let Some(addl_lua_paths) = addl_base_lua_paths {
			let mut paths: Vec<String> = Vec::new();
			for path in addl_lua_paths {
				paths.push(format!("{path}/lua/?.lua;{path}/lua/?/init.lua"));
			}
			if let Ok(lua_package) = globals.get::<Table>("package") {
				let path: String = lua_package.get("path")?;
				let new_path = format!("{};{path}", paths.join(";"));
				lua_package.set("path", new_path)?;
			}
		}

		// Return the updated scope table
		Ok(scope)
	}
}

// region:    --- Init Globals

fn init_package_path(lua: &Lua, runtime_context: &RuntimeContext) -> Result<()> {
	let globals = lua.globals();

	let package: Table = globals.get("package")?;
	// example of a default: "/usr/local/share/lua/5.4/?.lua;/usr/local/share/lua/5.4/?/init.lua;/usr/local/lib/lua/5.4/?.lua;/usr/local/lib/lua/5.4/?/init.lua;./?.lua;./?/init.lua"
	let path: String = package.get("path")?;

	let devai_dir = runtime_context.dir_context().devai_dir();

	// compute the additional paths

	// The .devai/custom/lua
	let custom_lua_dir = devai_dir.get_lua_custom_dir()?;
	let mut addl_paths = format!("{custom_lua_dir}/?.lua;{custom_lua_dir}/?/init.lua");

	// The eventual ~/.devai-base/custom/lua
	if let Some(base_lua_dir) = get_devai_base_dir().and_then(|base_dir| base_dir.join(CUSTOM_LUA_DIR).ok()) {
		if base_lua_dir.exists() {
			addl_paths = format!("{addl_paths};{base_lua_dir}/?.lua;{base_lua_dir}/?/init.lua");
		}
	}

	let new_path = format!("{addl_paths};{path}");
	package.set("path", new_path)?;

	Ok(())
}

fn init_print(lua: &Lua) -> Result<()> {
	let globals = lua.globals();

	globals.set(
		"print",
		lua.create_function(|_, args: mlua::Variadic<Value>| {
			let output: Vec<String> = args
				.iter()
				.map(|arg| match arg {
					Value::String(s) => s.to_str().map(|s| s.to_string()).unwrap_or_default(),
					Value::Number(n) => n.to_string(),
					Value::Integer(n) => n.to_string(),
					Value::Boolean(b) => b.to_string(),
					_ => "<unsupported value for print args>".to_string(),
				})
				.collect();

			let text = output.join("\t"); // Mimics Lua's `print` by joining args with tabs
			get_hub().publish_sync(HubEvent::LuaPrint(text.into()));
			Ok(())
		})?,
	)?;

	Ok(())
}

// endregion: --- Init Globals

// region:    --- init_utils

/// Just a convenient macro to init/set the lua modules
/// Will generate the code below for the name 'git'
/// ```rust
/// let git = utils_git::init_module(lua, runtime_context)?;
/// table.set("git", git)
/// ```
macro_rules! init_and_set {
    ($table:expr, $lua:expr, $runtime_context:expr, $($name:ident),*) => {
        paste::paste! {
            $(
                let $name = [<utils_ $name>]::init_module($lua, $runtime_context)?;
                $table.set(stringify!($name), $name)?;
            )*
        }
    };
}

/// Module builders
fn init_utils(lua: &Lua, runtime_context: &RuntimeContext) -> Result<()> {
	let table = lua.create_table()?;

	init_and_set!(
		table,
		lua,
		runtime_context,
		// -- The lua module names that refers to utils_...
		file,
		git,
		web,
		text,
		rust,
		path,
		md,
		json,
		html,
		cmd
	);

	let globals = lua.globals();
	globals.set("utils", table)?;
	Ok(())
}

// endregion: --- init_utils

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::SANDBOX_01_DIR;
	use crate::run::Runtime;
	use simple_fs::ensure_dir;
	use std::path::Path;

	/// Test if custom scope and global lua utils `math` work.
	#[tokio::test]
	async fn test_lua_engine_eval_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let engine = LuaEngine::new(runtime.context().clone())?;
		let fx_script = r#"
local square_root = math.sqrt(25)
return "Hello " .. my_name .. " - " .. square_root		
		"#;

		// -- Exec
		let scope = engine.create_table()?;
		scope.set("my_name", "Lua World")?;
		let res = engine.eval(fx_script, Some(scope), None)?;

		// -- Check
		let res = serde_json::to_value(res)?;
		let res = res.as_str().ok_or("Should be string")?;
		assert_eq!(res, "Hello Lua World - 5.0");
		Ok(())
	}

	/// Test if the `utils.file.load` works
	#[tokio::test]
	async fn test_lua_engine_eval_file_load_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let engine = LuaEngine::new(runtime.context().clone())?;
		let fx_script = r#"
local file = utils.file.load("other/hello.txt")
return "Hello " .. my_name .. " - " .. file.content		
		"#;

		// -- Exec
		let scope = engine.create_table()?;
		scope.set("my_name", "Lua World")?;
		let res = engine.eval(fx_script, Some(scope), None)?;

		// -- Check
		let res = serde_json::to_value(res)?;
		let res = res.as_str().ok_or("Should be string")?;
		assert_eq!(res, "Hello Lua World - hello from the other/hello.txt");

		Ok(())
	}

	/// Test if the `utils.file.load` works
	#[tokio::test]
	async fn test_lua_engine_eval_require_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		ensure_dir("tests-data/sandbox-01/.devai/custom/lua")?;
		std::fs::copy(
			Path::new(SANDBOX_01_DIR).join("other/demo.lua"),
			"tests-data/sandbox-01/.devai/custom/lua/demo.lua",
		)?;
		let engine = LuaEngine::new(runtime.context().clone())?;
		let fx_script = r#"
local demo = require("demo")
return "demo.name_one is " .. "'" .. demo.name_one .. "'"
		"#;

		// -- Exec
		let res = engine.eval(fx_script, None, None)?;

		// -- Check
		let res = serde_json::to_value(res)?;
		let res = res.as_str().ok_or("Should be string")?;
		assert_eq!(res, "demo.name_one is 'Demo One'");

		Ok(())
	}
}

// endregion: --- Tests
