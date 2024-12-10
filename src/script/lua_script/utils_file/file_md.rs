use crate::run::{PathResolver, RuntimeContext};
use crate::script::lua_script::helpers::to_vec_of_strings;
use crate::support::md::MdSectionIter;
use mlua::{IntoLua, Lua, Value};

/// ## Lua Documentation
///
/// Return the first FileMeta or Nil
///
/// ```lua
/// let all_summary_section = utils.file.list("doc/readme.md", "# Summary");
/// ```
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
	headings: Option<Value>,
) -> mlua::Result<Value> {
	let headings = headings
		.map(|headings| to_vec_of_strings(headings, "file::load_md_sections headings argument"))
		.transpose()?;
	let headings: Option<Vec<&str>> = headings.as_deref().map(|vec| {
		// Create a slice of string references directly from the vector.
		vec.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
	});

	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;

	let sec_iter = MdSectionIter::from_path(path, headings.as_deref())?;
	let sections = sec_iter.collect::<Vec<_>>();
	let res = sections.into_lua(lua)?;

	Ok(res)
}

pub(super) fn file_load_md_split_first(lua: &Lua, ctx: &RuntimeContext, path: String) -> mlua::Result<Value> {
	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;

	let mut sec_iter = MdSectionIter::from_path(path, None)?;
	let split_first = sec_iter.split_first();

	let res = split_first.into_lua(lua)?;

	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

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

	#[tokio::test]
	async fn test_lua_file_load_md_split_first_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/md-sections.md";

		// -- Exec
		let res = run_reflective_agent(
			&format!(r##"return utils.file.load_md_split_first("{fx_path}")"##),
			None,
		)
		.await?;

		// -- Check
		// check before
		let before = res.x_get_str("/before")?;
		assert_eq!(before, "", "before should be empty");
		// check first heading
		assert_eq!(
			res.x_get_str("/first/heading_content")?,
			"",
			"heading_content should be empty"
		);
		assert_eq!(res.x_get_i64("/first/heading_level")?, 0, "heading level should be 0");
		// check first content
		let content = res.x_get_str("/first/content")?;
		assert_contains(content, "Some early text");
		assert_contains(content, "- and more early text");
		assert_not_contains(content, "# Heading 1");
		assert_not_contains(content, "Some heading-1-content");
		// check the after
		let after = res.x_get_str("/after")?;
		assert_contains(after, "# Heading 1");
		assert_contains(after, "Some heading-1-content");
		assert_contains(after, "# Heading three");

		Ok(())
	}

	// other/md-sections.md
}

// endregion: --- Tests
