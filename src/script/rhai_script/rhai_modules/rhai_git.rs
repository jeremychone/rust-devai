//! Defines the `git` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//!
//! The `git` module exposes functions that call `git` commands.
//!
//! ### Functions
//! * `git::restore(file_path: string) -> string`

use crate::hub::get_hub;
use crate::run::RuntimeContext;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

pub fn rhai_module(runtime_context: &RuntimeContext) -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	let ctx = runtime_context.clone();
	FuncRegistration::new("restore")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str| git_restore(&ctx, path));

	module
}

// region:    --- Functions

/// ## RHAI Documentation
/// ```rhai
/// git::restore(file_path: string) -> string
/// ```
///
/// Calls `git restore {file_path}` and returns the output (stdout) of that
/// call.
/// The current_dir will be set at the devai_parent_dir as all relative rhai script context
///
/// # Arguments
///
/// * `file_path` - (required) A `String` containing the path to the file to be restored.
///
/// # Returns
///
/// A `String` containing the output of the `git restore` command.
fn git_restore(ctx: &RuntimeContext, path: &str) -> RhaiResult {
	let output = std::process::Command::new("git")
		.current_dir(ctx.dir_context().devai_parent_dir())
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
