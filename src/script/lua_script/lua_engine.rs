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
	pub fn eval(&self, script: &str, scope: Option<Table>) -> Result<Value> {
		let lua = &self.lua;

		let chunck = lua.load(script);

		let chunck = if let Some(scope) = scope {
			let env = self.upgrade_scope(scope)?;
			chunck.set_environment(env)
		} else {
			chunck
		};

		let res = chunck.eval::<Value>();
		let res = res?;
		let res = match res {
			// This is when we d with pcall(...), see test_lua_json_parse_invalid
			Value::Error(err) => {
				// for now we take the last
				// TODO: We need to handle those error better.
				if let Some(last) = err.chain().last() {
					// for now, sending back the same
					if let Some(crate_error) = last.downcast_ref::<Error>() {
						return Err(Error::custom(crate_error.to_string()));
					} else {
						// the TableError falls here
						return Err(Error::custom(last.to_string()));
					}
				} else {
					return Err(Error::cc("Lua error chain - ", err));
				}
			}
			res => res,
		};

		Ok(res)
	}

	pub fn create_table(&self) -> Result<Table> {
		let res = self.lua.create_table()?;
		Ok(res)
	}

	pub fn serde_to_lua_value(&self, val: serde_json::Value) -> Result<Value> {
		let res = self.lua.to_value(&val)?;
		Ok(res)
	}

	// pub fn to_lua_value(&self, val: impl IntoLua) -> Result<Value> {
	// 	let res = val.into_lua(&self.lua)?;
	// 	Ok(res)
	// }

	/// Upgrade a custom scope to full scope with all of the globals added.
	fn upgrade_scope(&self, scope: Table) -> Result<Table> {
		// Get the globals table
		let globals = self.lua.globals();

		// Iterate over globals and add them to the scope table
		for pair in globals.pairs::<Value, Value>() {
			let (key, value) = pair?;
			scope.set(key, value)?; // Add each global to the scope table
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
	use crate::run::Runtime;

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
		let res = engine.eval(fx_script, Some(scope))?;

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
		let res = engine.eval(fx_script, Some(scope))?;

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
		let engine = LuaEngine::new(runtime.context().clone())?;
		let fx_script = r#"
local demo_one = require("demo_one")
return "demo_one.name_one is " .. "'" .. demo_one.name_one .. "'"
		"#;

		// -- Exec
		let res = engine.eval(fx_script, None)?;

		// -- Check
		let res = serde_json::to_value(res)?;
		let res = res.as_str().ok_or("Should be string")?;
		assert_eq!(res, "demo_one.name_one is 'Demo One'");

		Ok(())
	}
}

// endregion: --- Tests
