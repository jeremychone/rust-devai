//! Defines the `cmd` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `cmd` module exposes functions to execute system commands.
//!
//! ### Functions
//! * `cmd::exec(cmd_name: string, args?: string | array) -> {stdout: string, stderr: string, exit: number}`

use crate::script::{dynamic_into_strings, DynaMap};
use crate::Error;
use rhai::plugin::RhaiResult;
use rhai::{Array, Dynamic, FuncRegistration, Module};
use std::process::Command;

pub fn rhai_module() -> Module {
	let mut module = Module::new();

	FuncRegistration::new("exec")
		.in_global_namespace()
		.set_into_module(&mut module, |cmd: &str| exec(cmd, None));

	FuncRegistration::new("exec")
		.in_global_namespace()
		.set_into_module(&mut module, |cmd: &str, args: Dynamic| exec(cmd, Some(args)));

	module
}

/// ## RHAI Documentation
///
/// Execute a system command with optional arguments.
///
/// ```
/// // API Signature
/// cmd::exec(cmd_name: string, args?: string | array) -> CmdResponse (throws: CmdException)
/// ```
///
/// The command will be executed using the system shell. Arguments can be provided as a single string
/// or an array of strings.
///
/// ### Example
/// ```
/// // Single string argument
/// let result = cmd::exec("echo", "hello world");
///
/// // Array of arguments
/// let result = cmd::exec("ls", ["-l", "-a"]);
/// ```
///
/// ### Returns (CmdResponse)
///
/// Returns when the command executes successfully (exit code 0).
///
/// ```
/// {
///   stdout: string,  // Standard output from the command
///   stderr: string,  // Standard error from the command
///   exit:   number   // Exit code (0 for success)
/// }
/// ```
///
/// ### Exception (CmdException)
///
/// ```
/// {
///   stdout: string | null,  // Standard output if available
///   stderr: string | null,  // Standard error if available
///   exit:   number | null,  // Exit code if available
///   error:  string         // Error message from command execution
/// }
/// ```
fn exec(cmd_name: &str, args: Option<Dynamic>) -> RhaiResult {
	let mut command = Command::new(cmd_name);

	// Handle optional arguments
	if let Some(args) = args {
		let args: Vec<String> = dynamic_into_strings(args, "command args")?;
		command.args(args);
	}

	match command.output() {
		Ok(output) => {
			let stdout = String::from_utf8_lossy(&output.stdout).to_string();
			let stderr = String::from_utf8_lossy(&output.stderr).to_string();
			let exit_code = output.status.code().unwrap_or(-1) as i64;

			if exit_code == 0 {
				let res: Dynamic = DynaMap::default()
					.insert("stdout", stdout)
					.insert("stderr", stderr)
					.insert("exit", exit_code)
					.into();
				Ok(res)
			} else {
				let res: Dynamic = DynaMap::default()
					.insert("stdout", stdout)
					.insert("stderr", stderr)
					.insert("exit", exit_code)
					.insert("error", format!("Command exited with non-zero status: {}", exit_code))
					.into();
				Err(Error::RhaiDynamic(res).into())
			}
		}
		Err(err) => {
			let res: Dynamic = DynaMap::default()
				.insert("stdout", ())
				.insert("stderr", ())
				.insert("exit", ())
				.insert("error", err.to_string())
				.into();
			Err(Error::RhaiDynamic(res).into())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::_test_support::run_reflective_agent;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_rhai_cmd_exec_echo_single_arg() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            return cmd::exec("echo", "hello world");
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_cmd_exec_echo_multiple_args() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            return cmd::exec("echo", ["hello", "world"]);
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_cmd_exec_invalid_command() -> Result<(), Box<dyn std::error::Error>> {
		let script = r#"
            try {
                cmd::exec("nonexistentcommand");
                return "should not reach here";
            } catch(ex) {
                return ex;
            }
        "#;

		let res = run_reflective_agent(script, None).await?;

		assert!(res.x_get_str("error")?.contains("No such file or directory"));

		Ok(())
	}
}
