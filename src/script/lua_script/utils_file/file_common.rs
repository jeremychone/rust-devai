use crate::hub::get_hub;
use crate::run::{PathResolver, RuntimeContext};
use crate::script::lua_script::helpers::to_vec_of_strings;
use crate::support::md::MdSectionIter;
use crate::types::{FileRecord, FileRef};
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};
use simple_fs::{ensure_file_dir, iter_files, list_files, ListOptions, SPath};
use std::fs::write;

/// ## Lua Documentation
///
/// Load a File Record object with its ontent
///
/// ```lua
/// local file = utils.file.load("doc/README.md")
/// -- file.content contains the text content of the file
/// ```
///
/// ### Returns
///
///
/// ```lua
/// -- FileRecord
/// {
///   path    = "doc/README.md",
///   content = "... text content of the file ...",
///   name    = "README.md",
///   stem    = "README",
///   ext     = "md",
/// }
/// ```
///
///
pub(super) fn file_load(lua: &Lua, ctx: &RuntimeContext, rel_path: String) -> mlua::Result<mlua::Value> {
	let base_path = ctx.dir_context().resolve_path("", PathResolver::DevaiParentDir)?;
	let rel_path = SPath::new(rel_path).map_err(Error::from)?;

	let file_record = FileRecord::load(base_path, rel_path)?;
	let res = file_record.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Save a File Content into a path
///
/// ```lua
/// utils.file.save("doc/README.md", "Some very cool documentation")
/// ```
///
/// ### Returns
///
/// Does not return anything
///
pub(super) fn file_save(_lua: &Lua, ctx: &RuntimeContext, rel_path: String, content: String) -> mlua::Result<()> {
	let path = ctx.dir_context().resolve_path(&rel_path, PathResolver::DevaiParentDir)?;
	ensure_file_dir(&path).map_err(Error::from)?;

	write(&path, content)?;

	get_hub().publish_sync(format!("-> Lua utils.file.save called on: {}", rel_path));

	Ok(())
}

/// ## Lua Documentation
///
/// List a set of file reference (no content) for a given glob
///
/// ```lua
/// let all_doc_file = utils.file.list("doc/**/*.md")
/// ```
///
///
/// ### Returns
///
/// ```lua
/// -- An array/table of FileRef
/// {
///   path    = "doc/README.md",
///   name    = "README.md",
///   stem    = "README",
///   ext     = "md",
/// }
/// ```
///
/// To get the content of files, needs iterate and load each
///
pub(super) fn file_list(lua: &Lua, ctx: &RuntimeContext, include_glob: String) -> mlua::Result<Value> {
	let base_path = ctx.dir_context().resolve_path("", PathResolver::DevaiParentDir)?;
	let sfiles = list_files(
		&base_path,
		Some(&[&include_glob]),
		Some(ListOptions::from_relative_glob(true)),
	)
	.map_err(Error::from)?;

	// Now, we put back the paths found relative to base_path
	let sfiles = sfiles
		.into_iter()
		.map(|f| f.diff(&base_path))
		.collect::<simple_fs::Result<Vec<SPath>>>()
		.map_err(|err| crate::Error::cc("Cannot list fiels to base", err))?;

	let file_refs: Vec<FileRef> = sfiles.into_iter().map(FileRef::from).collect();
	let res = file_refs.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Return the first FileRef or Nil
///
/// ```lua
/// let first_doc_file = utils.file.first("doc/**/*.md")
/// ```
///
///
/// ### Returns
///
/// ```lua
/// -- FileRef or Nil
/// {
///   path    = "doc/README.md",
///   name    = "README.md",
///   stem    = "README",
///   ext     = "md",
/// }
/// ```
///
/// To get the file record with .content, do
///
/// ```lua
/// let file = utils.file.load(file_ref.path)
/// ```
pub(super) fn file_first(lua: &Lua, ctx: &RuntimeContext, include_glob: String) -> mlua::Result<Value> {
	let base_path = ctx.dir_context().resolve_path("", PathResolver::DevaiParentDir)?;
	let mut sfiles = iter_files(
		&base_path,
		Some(&[&include_glob]),
		Some(ListOptions::from_relative_glob(true)),
	)
	.map_err(Error::from)?;

	let Some(sfile) = sfiles.next() else {
		return Ok(Value::Nil);
	};

	let sfile = sfile
		.diff(&base_path)
		.map_err(|err| Error::cc("Cannot diff with base_path", err))?;

	let res = FileRef::from(sfile).into_lua(lua)?;

	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent, SANDBOX_01_DIR};
	use std::path::Path;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_load_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "./agent-script/agent-hello.devai";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return utils.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(res.x_get_str("content")?, "from agent-hello.devai");
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_str("name")?, "agent-hello.devai");

		Ok(())
	}

	/// Note: need the multi-thread, because save do a `get_hub().publish_sync`
	///       which does a tokio blocking (requiring multi thread)
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_dest_path = "./.tmp/test_file_save_simple_ok/agent-hello.devai";
		let fx_content = "hello from test_file_save_simple_ok";

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return utils.file.save("{fx_dest_path}", "{fx_content}");"#),
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
	async fn test_lua_file_list_glob_direct() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return utils.file.list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res);
		assert_eq!(res_paths.len(), 2, "result length");
		assert_contains(&res_paths, "file-01.txt");
		assert_contains(&res_paths, "file-02.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return utils.file.list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res);
		assert_eq!(res_paths.len(), 2, "result length");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/agent-hello-3.devai");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/agent-hello-3.devai");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_first_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-2.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return utils.file.first("{glob}");"#), None).await?;

		// -- Check
		// let res_paths = to_res_paths(&res);
		assert_eq!(res.x_get_str("name")?, "agent-hello-2.devai");
		assert_eq!(res.x_get_str("path")?, "sub-dir-a/agent-hello-2.devai");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_first_not_found() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-not-a-thing.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return utils.file.first("{glob}")"#), None).await?;

		// -- Check
		assert_eq!(res, serde_json::Value::Null, "Should have returned null");

		Ok(())
	}

	// region:    --- Support for Tests

	fn to_res_paths(res: &serde_json::Value) -> Vec<&str> {
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
