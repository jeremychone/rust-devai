//! Defines the `file` module, used in the rhai engine.
//!
//! ---
//!
//! ## RHAI documentation
//! The `file` module exposes functions used to read, write, or modify files.
//!
//! ### Functions
//! * `file::load_md_sections(path: string, headings: array) -> array`

use crate::run::{PathResolver, RuntimeContext};
use crate::script::rhai_script::dynamic_helpers::dynamic_into_strings;
use crate::script::IntoDynamic;
use crate::support::md::read_file_md_sections;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, Module};

pub fn rhai_module(runtime_context: &RuntimeContext) -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	let ctx = runtime_context.clone();
	FuncRegistration::new("load_md_sections")
		.in_global_namespace()
		.set_into_module(&mut module, move |path: &str, headings: Dynamic| {
			load_md_sections(&ctx, path, headings)
		});

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// file::load_md_sections(path: string, headings: array) -> array
/// ```
///
/// Loads specific sections from a markdown file based on the provided headings.
///
/// This function reads a markdown file and extracts sections that match the specified headings.
///
/// # Arguments
///
/// * `path` - (required) A `String` representing the path to the markdown file.
/// * `headings` - (required) An `Array` of `String` headings to search for in the markdown file.
///
/// # Returns
///
/// An `Array` of sections, each containing the heading and its content.
fn load_md_sections(ctx: &RuntimeContext, path: &str, headings: Dynamic) -> RhaiResult {
	// Convert `headings` to a `Vec<String>`
	let headings: Vec<String> = dynamic_into_strings(headings, "file::load_md_sections headings argument")?;

	let headings = headings.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

	let path = ctx.dir_context().resolve_path(path, PathResolver::DevaiParentDir)?;

	let sections = read_file_md_sections(path, &headings)?;

	Ok(sections.into_dynamic())
}

// endregion: --- Rhai Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent};
	use value_ext::JsonValueExt;

	#[tokio::test]
	async fn test_file_load_md_sections_heading_1_top_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/md-sections.md";

		// -- Exec
		let mut res = run_reflective_agent(
			&format!(r##"return file::load_md_sections("{fx_path}", ["# Heading 1   "]);"##),
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
		let heading_content = first_item.x_get_str("heading_content")?;
		let heading_level = first_item.x_get_i64("heading_level")?;
		assert_contains(heading_content, "# Heading 1");
		assert_eq!(heading_level, 1, "heading level");
		assert_contains(content, "heading-1-content");
		assert_contains(content, "sub heading 1-a");
		assert_not_contains(content, "heading-1-a-blockquote");
		assert_not_contains(content, "content-2");

		Ok(())
	}
}

// endregion: --- Tests
