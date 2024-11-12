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
use crate::script::rhai_script::dynamic_helpers::dynamic_into_strings;
use crate::script::IntoDynamic;
use crate::support::md::read_file_md_sections;
use crate::types::{FileRecord, FileRef};
use crate::{Error, Result};
use rhai::plugin::RhaiResult;
use rhai::{Array, Dynamic, EvalAltResult, FuncRegistration, Module};
use simple_fs::{ensure_file_dir, iter_files, list_files, ListOptions, SPath};
use std::fs::write;

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

fn load_md_sections(ctx: &RuntimeContext, path: &str, mut headings: Dynamic) -> RhaiResult {
	// headings.is_string()
	// Convert `headings` to a `Vec<String>`

	// -- Compute the headings
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

	use crate::_test_support::{assert_contains, assert_not_contains, run_reflective_agent, SANDBOX_01_DIR};
	use serde_json::Value;
	use std::path::Path;
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
