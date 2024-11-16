//! Defines the `path` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `path` module exposes functions used to interact with file paths.
//!
//! ### Functions
//! * `path::exists(path: string) -> bool`
//! * `path::is_file(path: string) -> bool`
//! * `path::is_dir(path: string) -> bool`
//! * `path::parent(path: string) -> string | void`

use crate::run::{PathResolver, RuntimeContext};
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, Module};
use std::path::Path;

pub fn rhai_module(runtime_context: &RuntimeContext) -> Module {
	// Create a module for path functions
	let mut module = Module::new();

	let ctx = runtime_context.clone();
	FuncRegistration::new("exists")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str| path_exists(&ctx, path));

	let ctx = runtime_context.clone();
	FuncRegistration::new("is_file")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str| path_is_file(&ctx, path));

	let ctx = runtime_context.clone();
	FuncRegistration::new("is_dir")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str| path_is_dir(&ctx, path));

	let ctx = runtime_context.clone();
	FuncRegistration::new("parent")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str| path_parent(&ctx, path));

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// path::exists(path: string) -> bool
/// ```
///
/// Checks if the specified path exists.
fn path_exists(ctx: &RuntimeContext, path: &str) -> RhaiResult {
	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;
	Ok(Dynamic::from(path.exists()))
}

/// ## RHAI Documentation
/// ```rhai
/// path::is_file(path: string) -> bool
/// ```
///
/// Checks if the specified path is a file.
fn path_is_file(ctx: &RuntimeContext, path: &str) -> RhaiResult {
	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;
	Ok(Dynamic::from(path.is_file()))
}

/// ## RHAI Documentation
/// ```rhai
/// path::is_dir(path: string) -> bool
/// ```
///
/// Checks if the specified path is a directory.
fn path_is_dir(ctx: &RuntimeContext, path: &str) -> RhaiResult {
	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;
	Ok(Dynamic::from(path.is_dir()))
}

/// ## RHAI Documentation
/// ```rhai
/// path::parent(path: string) -> string | void
/// ```
///
/// Returns the parent directory of the specified path, or null/void if there is no parent.
/// (follows the Rust Path::parent(&self) logic)
fn path_parent(_ctx: &RuntimeContext, path: &str) -> RhaiResult {
	match Path::new(path).parent() {
		Some(parent) => match parent.to_str() {
			Some(parent_str) => Ok(Dynamic::from(parent_str.to_string())),
			None => Ok(Dynamic::UNIT),
		},
		None => Ok(Dynamic::UNIT),
	}
}

// endregion: --- Rhai Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	//! NOTE 1: Here we are testing these functions in the context of an agent to ensure they work in that context.
	//!         A more purist approach would be to test the Rhai functions in a blank Rhai engine, but the net value of testing
	//!         them in the context where they will run is higher. Height wins.
	//!
	//! NOTE 2: All the tests here are with run_reflective_agent that have the tests-data/sandbox-01 as current dir.
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;

	#[tokio::test]
	async fn test_rhai_path_exists_true() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./agent-script/agent-hello.devai",
			"agent-script/agent-hello.devai",
			"./sub-dir-a/agent-hello-2.devai",
			"sub-dir-a/agent-hello-2.devai",
			"sub-dir-a/",
			"sub-dir-a",
			"./sub-dir-a/",
			"./sub-dir-a/../",
			"./sub-dir-a/..",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::exists("{path}");"#), None).await?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should exists"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_path_exists_false() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./no file .rs",
			"some/no-file.md",
			"./s do/",
			"no-dir/at/all",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::exists("{path}");"#), None).await?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT exists"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_path_is_file_true() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./agent-script/agent-hello.devai",
			"agent-script/agent-hello.devai",
			"./sub-dir-a/agent-hello-2.devai",
			"sub-dir-a/agent-hello-2.devai",
			"sub-dir-a/../agent-script/agent-hello.devai",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_file("{path}");"#), None).await?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should be is_file"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_path_is_file_false() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./no-file",
			"no-file.txt",
			"sub-dir-a/",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_file("{path}");"#), None).await?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be is_file"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_path_is_dir_true() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./sub-dir-a",
			"sub-dir-a",
			"./sub-dir-a/..",
			// Note: below does not work for now becsuse some-other-dir does not exists. Might want to use clean.
			// "./sub-dir-a/some-other-dir/..",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_dir("{path}");"#), None).await?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should be is_dir"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_path_is_dir_false() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./agent-hello.devai",
			"agent-hello.devai",
			"./sub-dir-a/agent-hello-2.devai",
			"./sub-dir-a/other-path",
			"nofile.txt",
			"./s rc/",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_dir("{path}");"#), None).await?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be is_dir"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_path_parent() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let paths = &[
			//
			("./agent-hello.devai", "."),
			("./", ""),
			(".", ""),
			("./sub-dir/file.txt", "./sub-dir"),
			("./sub-dir/file", "./sub-dir"),
			("./sub-dir/", "."),
			("./sub-dir", "."),
		];

		// -- Exec & Check
		for (path, expected) in paths {
			let res = run_reflective_agent(&format!(r#"return path::parent("{path}");"#), None).await?;
			let res = res.as_str().ok_or("Should be a string")?;
			assert_eq!(res, *expected);
		}

		Ok(())
	}
}

// endregion: --- Tests
