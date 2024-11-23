use crate::hub::get_hub;
use crate::run::RuntimeContext;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let ctx = runtime_context.clone();
	let git_restore_fn = lua.create_function(move |lua, (path,): (String,)| git_restore(lua, &ctx, path))?;

	table.set("restore", git_restore_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Will do a `git restore path`
///
/// ```lua
/// utils.git.restore("src/main.rs")
/// ```
///
/// NOTE: The git command will be with the working dir as the devai_parent_dir to be consistent with the other
///
///
fn git_restore(lua: &Lua, ctx: &RuntimeContext, path: String) -> mlua::Result<Value> {
	let output = std::process::Command::new("git")
		.current_dir(ctx.dir_context().devai_parent_dir())
		.arg("restore")
		.arg(&path)
		.output()
		.expect("Failed to execute command");

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stderr = String::from_utf8_lossy(&output.stderr);

	if !stderr.is_empty() {
		get_hub().publish_sync(format!("stderr: {}", stderr));
		return Err(Error::cc(format!("'git restore {path}' failed"), stderr).into());
	}

	stdout.into_lua(lua)
}
