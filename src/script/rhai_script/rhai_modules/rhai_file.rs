//! Defines the `file` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `file` module exposes functions used to read, write, or modify files.
//!
//! ### Functions
//! * `file::load(file_path: string) -> FileRecord`
//! * `file::save(file_path: string, content: string)`
//! * `file::list(glob: string) -> Vec<FileRef>`

use crate::hub::get_hub;
use crate::run::{PathResolver, RuntimeContext};
use crate::types::{FileRecord, FileRef};
use crate::{Error, Result};
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, EvalAltResult, FuncRegistration, Module};
use simple_fs::{ensure_file_dir, iter_files, list_files, ListOptions, SPath};
use std::fs::write;

pub fn rhai_module(runtime_context: &RuntimeContext) -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	let ctx = runtime_context.clone();
	FuncRegistration::new("load")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str| load(&ctx, path));

	let ctx = runtime_context.clone();
	FuncRegistration::new("save")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str, content: &str| save(&ctx, path, content));

	let ctx = runtime_context.clone();
	FuncRegistration::new("list")
		.in_global_namespace()
		.set_into_module(&mut module, move |include_glob: &str| {
			list_with_glob(&ctx, include_glob)
		});

	let ctx = runtime_context.clone();
	FuncRegistration::new("first")
		.in_global_namespace()
		.set_into_module(&mut module, move |include_glob: &str| {
			first_with_glob(&ctx, include_glob)
		});

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// file::list(glob: string) -> Vec<FileRef>
/// ```
///
/// Expands `glob`, returning a list of all matching file paths along with
/// helpful metadata.
///
/// Note: The `FileRef`do not load the content,
///       so it has .path, .name, .stem, .ext, but does NOT have .content
///
/// To get the list of files with their content, do as follow:
///
/// ```rhai
/// let file_list = file::list("your-glob-pattern");
/// let file_records = file_list.map(|file_ref| file::load(file_ref.path));
/// ```
///
fn list_with_glob(ctx: &RuntimeContext, include_glob: &str) -> RhaiResult {
	let base_path = ctx.dir_context().resolve_path("", PathResolver::DevaiParentDir)?;
	let sfiles = list_files(
		&base_path,
		Some(&[include_glob]),
		Some(ListOptions::from_relative_glob(true)),
	)
	.map_err(|err| {
		EvalAltResult::ErrorRuntime(
			format!("Failed to list files with glob: {include_glob}. Cause: {}", err).into(),
			rhai::Position::NONE,
		)
	})?;

	// Now, we put back the paths found relative to base_path
	let sfiles = sfiles
		.into_iter()
		.map(|f| f.diff(&base_path))
		.collect::<simple_fs::Result<Vec<SPath>>>()
		.map_err(|err| crate::Error::cc("Cannot list fiels to base", err))?;

	let file_refs: Vec<FileRef> = sfiles.into_iter().map(FileRef::from).collect();
	let file_dynamics: Vec<Dynamic> = file_refs.into_iter().map(FileRef::into_dynamic).collect();
	let res_dynamic = Dynamic::from_array(file_dynamics);

	Ok(res_dynamic)
}

/// ## RHAI Documentation
/// ```rhai
/// file::first(glob: string) -> FileRef | null
/// ```
///
/// Expands `glob`, returning the first matching file path along with
///
/// Note: The `FileRef` has .path, .name, .stem, .ext, but does NOT have .content
///
/// To get the list of files with their content, do as follow:
///
/// ```rhai
/// let my_file = file::first("src/**/my_file.rs");
/// if my_file {
///   return #{
///      my_file: file::load(my_file.path)
///   }
/// }
/// } else {
///   return devai::skip("no file found");
/// }
///
/// ```
///
fn first_with_glob(ctx: &RuntimeContext, include_glob: &str) -> RhaiResult {
	let base_path = ctx.dir_context().resolve_path("", PathResolver::DevaiParentDir)?;
	let mut sfiles = iter_files(
		&base_path,
		Some(&[include_glob]),
		Some(ListOptions::from_relative_glob(true)),
	)
	.map_err(|err| {
		EvalAltResult::ErrorRuntime(
			format!("Failed to list files with glob: {include_glob}. Cause: {}", err).into(),
			rhai::Position::NONE,
		)
	})?;

	let Some(sfile) = sfiles.next() else {
		return Ok(Dynamic::UNIT);
	};

	let sfile = sfile
		.diff(&base_path)
		.map_err(|err| Error::cc("Cannot diff with base_path", err))?;

	let file_ref_dynamic = FileRef::from(sfile).into_dynamic();

	Ok(file_ref_dynamic)
}

/// ## RHAI Documentation
/// ```rhai
/// file::load(file_path: string) -> FileRecord
/// ```
///
/// Reads the file specified by `path`, returning the contents of the file
/// along with helpful metadata.
fn load(ctx: &RuntimeContext, rel_path: &str) -> RhaiResult {
	let base_path = ctx.dir_context().resolve_path("", PathResolver::DevaiParentDir)?;
	let rel_path = SPath::new(rel_path).map_err(Error::from)?;

	let file_record = FileRecord::load(base_path, rel_path);

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
/// file::save(file_path: string, content: string)
/// ```
///
/// Writes `content` to the specified `file_path`.
fn save(ctx: &RuntimeContext, file_path: &str, content: &str) -> RhaiResult {
	fn file_save_inner(ctx: &RuntimeContext, file_path: &str, content: &str) -> Result<()> {
		let path = ctx.dir_context().resolve_path(file_path, PathResolver::DevaiParentDir)?;
		ensure_file_dir(&path)?;
		write(&path, content)?;

		get_hub().publish_sync(format!("-> Rhai file::save called on: {}", file_path));
		Ok(())
	}

	match file_save_inner(ctx, file_path, content) {
		Ok(_) => Ok(().into()),
		Err(err) => Err(Box::new(EvalAltResult::ErrorRuntime(
			format!(" Rhai file::save Failed for file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

// endregion: --- Rhai Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, run_reflective_agent, SANDBOX_01_DIR};
	use serde_json::Value;
	use std::path::Path;
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_file_load_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "./agent-script/agent-hello.md";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return file::load("{fx_path}");"#), None).await?;

		// -- Check
		assert_contains(res.x_get_str("content")?, "from agent-hello.md");
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_str("name")?, "agent-hello.md");

		Ok(())
	}

	/// Note: need the multi-thread, because save do a `get_hub().publish_sync`
	///       which does a tokio blocking (requiring multi thread)
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_file_save_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_dest_path = "./.test_file_save_simple_ok/agent-hello.md";
		let fx_content = "hello from test_file_save_simple_ok";

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return file::save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await?;

		// -- Check
		let dest_path = Path::new(SANDBOX_01_DIR).join(fx_dest_path);
		let file_content = std::fs::read_to_string(dest_path)?;
		assert_eq!(file_content, fx_content);

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_file_list_glob_direct() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return file::list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res);
		assert_eq!(res_paths.len(), 2, "result length");
		assert_contains(&res_paths, "file-01.txt");
		assert_contains(&res_paths, "file-02.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_file_list_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return file::list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res);
		assert_eq!(res_paths.len(), 2, "result length");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/agent-hello-3.md");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/agent-hello-3.md");

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_file_first_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-2.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return file::first("{glob}");"#), None).await?;

		// -- Check
		// let res_paths = to_res_paths(&res);
		assert_eq!(res.x_get_str("name")?, "agent-hello-2.md");
		assert_eq!(res.x_get_str("path")?, "sub-dir-a/agent-hello-2.md");

		Ok(())
	}

	#[tokio::test]
	async fn test_rhai_file_first_not_found() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-not-a-thing.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return file::first("{glob}");"#), None).await?;

		// -- Check
		assert_eq!(res, Value::Null, "Should have returned null");

		Ok(())
	}

	// region:    --- Support for Tests

	fn to_res_paths(res: &Value) -> Vec<&str> {
		res.as_array()
			.ok_or("should have array of path")
			.unwrap()
			.iter()
			.map(|v| v.x_get_as::<&str>("path").unwrap_or_default())
			.collect::<Vec<&str>>()
	}

	// endregion: --- Support for Tests
}

// endregion: --- Tests
