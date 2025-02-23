//! Defines the `git` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua Documentation
//! The `git` module exposes functions for performing Git operations.
//!
//! ### Functions
//! * `utils.git.restore(path: string) -> string | table`

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

// region: --- Lua Functions

/// ## Lua Documentation
/// ```lua
/// utils.git.restore(path: string) -> string | table
/// ```
/// Executes a `git restore` command in the workspace directory using the given file path.
///
/// ### Returns
/// Returns the standard output as a string if the command is successful.
///
/// ### Exception
/// Throws an error if the command's stderr output is not empty.
///
/// ### Example
/// ```lua
/// local result = utils.git.restore("src/main.rs")
/// print(result)
/// ```
fn git_restore(lua: &Lua, ctx: &RuntimeContext, path: String) -> mlua::Result<Value> {
	let output = std::process::Command::new("git")
		.current_dir(ctx.dir_context().wks_dir())
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

// endregion: --- Lua Functions
