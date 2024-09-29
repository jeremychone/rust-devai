//! Defines the `file` module, used in the rhai engine. 
//! 
//! ---
//! 
//! ## RHAI documentation
//! The `file` module exposes functions used to read, write, or modify files.
//! 
//! ### Functions
//! * `load(file_path: string) -> FileRecord`
//! * `save(file_path: string, content: string)`
//! * `list(glob: string) -> Vec<FileRef>`

use crate::types::{FileRecord, FileRef};
use crate::Result;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, EvalAltResult, FuncRegistration, Module};
use simple_fs::{ensure_file_dir, list_files};
use std::fs::write;
use std::path::Path;

/// Define the `file` module and register functions
pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("load")
		.in_global_namespace()
		.set_into_module(&mut module, load);

	FuncRegistration::new("save")
		.in_global_namespace()
		.set_into_module(&mut module, save);

	FuncRegistration::new("list")
		.in_global_namespace()
		.set_into_module(&mut module, list_with_glob);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// list(glob: string) -> Vec<FileRef>
/// ```
/// 
/// Expands `glob`, returning a list of all matching file paths along with
/// helpful metadata.
fn list_with_glob(include_glob: &str) -> RhaiResult {
	let sfiles = list_files("./", Some(&[include_glob]), None).map_err(|err| {
		EvalAltResult::ErrorRuntime(
			format!("Failed to list files with glob: {include_glob}. Cause: {}", err).into(),
			rhai::Position::NONE,
		)
	})?;

	let file_refs: Vec<FileRef> = sfiles.into_iter().map(FileRef::from).collect();
	let file_dynamics: Vec<Dynamic> = file_refs.into_iter().map(FileRef::into_dynamic).collect();
	let res_dynamic = Dynamic::from_array(file_dynamics);

	Ok(res_dynamic)
}

/// ## RHAI Documentation
/// ```rhai
/// load(file_path: string) -> FileRecord
/// ```
/// 
/// Reads the file specified by `path`, returning the contents of the file
/// along with helpful metadata.
fn load(file_path: &str) -> RhaiResult {
	let file_record = FileRecord::new(file_path);
	match file_record {
		Ok(file) => Ok(file.into()),
		Err(err) => Err(Box::new(EvalAltResult::ErrorRuntime(
			format!("Failed to load file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

/// ## RHAI Documentation
/// ```rhai
/// save(file_path: string, content: string)
/// ```
/// 
/// Writes `content` to the specified `file_path`.
fn save(file_path: &str, content: &str) -> RhaiResult {
	fn file_save_inner(file_path: &str, content: &str) -> Result<()> {
		// let sfile = SFile::from_path(file_path)?;
		let path = Path::new(file_path);
		ensure_file_dir(path)?;
		write(path, content)?;
		println!("\n-- Rhai file::save called on: {}\n", file_path);
		Ok(())
	}

	match file_save_inner(file_path, content) {
		Ok(_) => Ok(().into()),
		Err(err) => Err(Box::new(EvalAltResult::ErrorRuntime(
			format!(" Rhai file::save Failed for file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

// endregion: --- Rhai Functions
