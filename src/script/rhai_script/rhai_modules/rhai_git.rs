//! Defines the `git` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//!
//! The `git` module exposes functions that call `git` commands.
//!
//! ### Functions
//! * `restore(file_path: string) -> string`

use crate::hub::get_hub;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("restore")
		.in_global_namespace()
		.set_into_module(&mut module, git_restore);

	module
}

// region:    --- Functions

/// ## RHAI Documentation
/// ```rhai
/// restore(file_path: string) -> string
/// ```
///
/// Calls `git restore {file_path}` and returns the output (stdout) of that
/// call.
fn git_restore(path: &str) -> RhaiResult {
	let output = std::process::Command::new("git")
		.arg("restore")
		.arg(path)
		.output()
		.expect("Failed to execute command");

	let stdout = String::from_utf8_lossy(&output.stdout);
	let stderr = String::from_utf8_lossy(&output.stderr);

	if !stderr.is_empty() {
		get_hub().publish_sync(format!("stderr: {}", stderr));
		return Err(stderr.into());
	}

	Ok(stdout.to_string().into())
}

// endregion: --- Functions
