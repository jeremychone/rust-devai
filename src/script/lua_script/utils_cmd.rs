//! Defines the `cmd` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `cmd` module exposes functions to execute system commands.
//!
//! ### Functions
//! * `utils.cmd.exec(cmd_name: string, args?: string | table) -> {stdout: string, stderr: string, exit: number}`

use crate::run::RuntimeContext;
use crate::script::lua_script::helpers::{make_table_external_error, to_vec_of_strings};
use mlua::{Lua, Table, Value};
use std::process::Command;

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> mlua::Result<Table> {
	let table = lua.create_table()?;

	let exec_fn = lua.create_function(cmd_exec)?;

	table.set("exec", exec_fn)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Execute a system command with optional arguments.
///
/// ```lua
/// -- API Signature
/// utils.cmd.exec(cmd_name: string, args?: string | table) -> CmdResponse
/// ```
///
/// The command will be executed using the system shell. Arguments can be provided as a single string
/// or a table of strings.
///
/// ### Example
/// ```lua
/// -- Single string argument
/// local result = utils.cmd.exec("echo", "hello world")
///
/// -- Table of arguments
/// local result = utils.cmd.exec("ls", {"-l", "-a"})
/// ```
///
/// ### Returns (CmdResponse)
///
/// Returns when the command executes successfully (exit code 0).
///
/// ```lua
/// {
///   stdout = string,  -- Standard output from the command
///   stderr = string,  -- Standard error from the command
///   exit   = number   -- Exit code (0 for success)
/// }
/// ```
///
/// ### Error
///
/// ```lua
/// {
///   stdout = string | nil,  -- Standard output if available
///   stderr = string | nil,  -- Standard error if available
///   exit   = number | nil,  -- Exit code if available
///   error  = string        -- Error message from command execution
/// }
/// ```
fn cmd_exec(lua: &Lua, (cmd_name, args): (String, Option<Value>)) -> mlua::Result<Value> {
	let mut command = Command::new(cmd_name);

	// Handle optional arguments
	if let Some(args) = args {
		let args = to_vec_of_strings(args, "command args")?;
		command.args(args);
	}

	match command.output() {
		Ok(output) => {
			let stdout = String::from_utf8_lossy(&output.stdout).to_string();
			let stderr = String::from_utf8_lossy(&output.stderr).to_string();
			let exit_code = output.status.code().unwrap_or(-1) as i64;

			let res = lua.create_table()?;
			res.set("stdout", stdout)?;
			res.set("stderr", stderr)?;
			res.set("exit", exit_code)?;

			if exit_code == 0 {
				Ok(Value::Table(res))
			} else {
				res.set("error", format!("Command exited with non-zero status: {}", exit_code))?;
				Err(make_table_external_error(res))
			}
		}
		Err(err) => {
			let res = lua.create_table()?;
			res.set("stdout", Value::Nil)?;
			res.set("stderr", Value::Nil)?;
			res.set("exit", Value::Nil)?;
			res.set("error", err.to_string())?;
			Err(make_table_external_error(res))
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::_test_support::run_reflective_agent;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_lua_cmd_exec_echo_single_arg() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            return utils.cmd.exec("echo", "hello world")
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_echo_multiple_args() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            return utils.cmd.exec("echo", {"hello", "world"})
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_invalid_command() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            local ok, err = pcall(function()
                return utils.cmd.exec("nonexistentcommand")
            end)
            return err
        "#;

		let Err(err) = run_reflective_agent(script, None).await else {
			return Err("Should have returned an error".into());
		};

		assert!(format!("{err}").contains("No such file or directory"));

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_invalid_no_handle() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"return utils.cmd.exec("nonexistentcommand")"#;

		let Err(err) = run_reflective_agent(script, None).await else {
			return Err("Should have returned an error".into());
		};

		assert!(format!("{err}").contains("No such file or directory"));

		Ok(())
	}
}
