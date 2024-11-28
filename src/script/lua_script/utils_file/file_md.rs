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
/// Return the first FileRef or Nil
///
/// ```lua
/// let all_summary_section = utils.file.list("doc/readme.md", "# Summary");
/// ```
///
///
/// ### Returns
///
/// ```lua
/// -- Array/Table of MdSection
/// {
///   content = "Content of the section",
///   -- heading is optional
///   heading = {content = "# Summary", level = 1, name = "Summary"},
/// }
/// ```
///
pub(super) fn file_load_md_sections(
	lua: &Lua,
	ctx: &RuntimeContext,
	path: String,
	headings: Value,
) -> mlua::Result<Value> {
	let headings: Vec<String> = to_vec_of_strings(headings, "file::load_md_sections headings argument")?;
	let headings = headings.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;

	let sec_iter = MdSectionIter::from_path(path, Some(&headings))?;
	let sections = sec_iter.collect::<Vec<_>>();
	let res = sections.into_lua(lua)?;

	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_lua_file_load_md_sections_heading_1_top_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/md-sections.md";

		// -- Exec
		let mut res = run_reflective_agent(
			&format!(r##"return utils.file.load_md_sections("{fx_path}", {{"# Heading 1   "}})"##),
			None,
		)
		.await?;

		// -- Check
		let first_item = res
			.as_array_mut()
			.ok_or("Res should be array")?
			.iter_mut()
			.next()
			.ok_or("Should have at least one item")?;

		let content = first_item.x_get_str("content")?;
		let heading_content = first_item.x_get_str("/heading_content")?;
		let heading_level = first_item.x_get_i64("/heading_level")?;
		let heading_name = first_item.x_get_str("/heading_name")?;
		assert_eq!(heading_level, 1, "heading level");
		// contains
		assert_contains(heading_content, "# Heading 1");
		assert_contains(heading_name, "Heading 1");
		assert_contains(content, "heading-1-content");
		assert_contains(content, "sub heading 1-a");
		assert_contains(content, "heading-1-a-blockquote");
		// not contains
		assert_not_contains(content, "content-2");
		assert_not_contains(content, "heading-2-blockquote");

		Ok(())
	}

	// other/md-sections.md
}

// endregion: --- Tests
