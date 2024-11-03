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

use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, Module};
use std::path::Path;

pub fn rhai_module() -> Module {
	// Create a module for path functions
	let mut module = Module::new();

	FuncRegistration::new("exists")
		.in_global_namespace()
		.set_into_module(&mut module, path_exists);

	FuncRegistration::new("is_file")
		.in_global_namespace()
		.set_into_module(&mut module, path_is_file);

	FuncRegistration::new("is_dir")
		.in_global_namespace()
		.set_into_module(&mut module, path_is_dir);

	FuncRegistration::new("parent")
		.in_global_namespace()
		.set_into_module(&mut module, path_parent);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// path::exists(path: string) -> bool
/// ```
///
/// Checks if the specified path exists.
fn path_exists(path: &str) -> RhaiResult {
	let exists = Path::new(path).exists();
	Ok(Dynamic::from(exists))
}

/// ## RHAI Documentation
/// ```rhai
/// path::is_file(path: string) -> bool
/// ```
///
/// Checks if the specified path is a file.
fn path_is_file(path: &str) -> RhaiResult {
	let is_file = Path::new(path).is_file();
	Ok(Dynamic::from(is_file))
}

/// ## RHAI Documentation
/// ```rhai
/// path::is_dir(path: string) -> bool
/// ```
///
/// Checks if the specified path is a directory.
fn path_is_dir(path: &str) -> RhaiResult {
	let is_dir = Path::new(path).is_dir();
	Ok(Dynamic::from(is_dir))
}

/// ## RHAI Documentation
/// ```rhai
/// path::parent(path: string) -> string | void
/// ```
///
/// Returns the parent directory of the specified path, or null/void if there is no parent.
fn path_parent(path: &str) -> RhaiResult {
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
	//! NOTE: Here we are testing these functions in the context of an agent to ensure they work in that context.
	//!       A more purist approach would be to test the Rhai functions in a blank Rhai engine, but the net value of testing
	//!       them in the context where they will run is higher. Height wins.
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;

	#[tokio::test]
	async fn test_rhai_path_exists_true() -> Result<()> {
		// -- Fixtures
		let paths = &[
			//
			"./",
			"./src/exec",
			"./src/exec/",
			"./src/main.rs",
			"src/main.rs",
			"Cargo.toml",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::exists("{path}");"#)).await?;
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
			"./src/main .rs",
			"src/lib.rs",
			"./s rc/",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::exists("{path}");"#)).await?;
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
			"./src/main.rs",
			"src/main.rs",
			"Cargo.toml",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_file("{path}");"#)).await?;
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
			"./src",
			"./",
			"./no-file.none",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_file("{path}");"#)).await?;
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
			"./src",
			"./",
			"./src/exec/",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_dir("{path}");"#)).await?;
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
			"./src/main.rs",
			"src/main.rs",
			"Cargo.toml",
			"./s rc/",
		];

		// -- Exec & Check
		for path in paths {
			let res = run_reflective_agent(&format!(r#"return path::is_dir("{path}");"#)).await?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be is_dir"
			);
		}

		Ok(())
	}
}

// endregion: --- Tests
