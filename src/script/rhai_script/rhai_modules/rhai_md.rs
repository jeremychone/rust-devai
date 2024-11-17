//! Defines the `md` module, used in the rhai engine.
//!
//! ## RHAI Documentation
//! The `md` module exposes functions that process markdown content. Useful for
//! processing LLM responses.
//!
//! ### Functions
//! * `md::extract_blocks_with_lang(md_content: string, lang_name: string) -> Vec<MdBlock>`
//! * `md::outer_block_content_or_raw(md_content: string) -> Vec<MdBlock>`

use crate::script::IntoDynamic;
use crate::support::md;
use crate::types::MdBlock;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("extract_blocks")
		.in_global_namespace()
		.set_into_module(&mut module, extract_blocks);

	FuncRegistration::new("extract_blocks")
		.in_global_namespace()
		.set_into_module(&mut module, extract_blocks_with_lang);

	FuncRegistration::new("outer_block_content_or_raw")
		.in_global_namespace()
		.set_into_module(&mut module, outer_block_content_or_raw);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// md::extract_blocks(md_content: &str) -> Vec<MdBlock>
/// ```
///
/// Return the list of markdown blocks with or without lang information
fn extract_blocks(md_content: &str) -> RhaiResult {
	let blocks: Vec<MdBlock> = md::MdBlocks::new(md_content, None).collect();
	Ok(blocks.into_dynamic())
}

/// ## RHAI Documentation
/// ```rhai
/// md::extract_blocks(md_content: &str, lang_name) -> Vec<MdBlock>
/// ```
///
/// Return the list of markdown blocks that match a given lang_name.
fn extract_blocks_with_lang(md_content: &str, lang_name: &str) -> RhaiResult {
	let blocks: Vec<MdBlock> = md::MdBlocks::new(md_content, Some(lang_name)).collect();
	Ok(blocks.into_dynamic())
}

/// ## RHAI Documentation
/// ```rhai
/// md::outer_block_content_or_raw(md_content: &str) -> Vec<MdBlock>
/// ```
///
/// Without fully parsing the markdown, this function attempts to extract the content from the first set of triple backticks
/// to the last set of triple backticks.
/// If no starting or ending triple backticks are found, it will return the raw content.
///
/// > Note: This is useful in the GenAI context because often LLMs return a top block (e.g., markdown, Rust)
/// >       which might have other ` ``` ` in the middle but should be interpreted as nested.
/// >       (GenAI does not seem to recognize the use of 6 backticks for top-level blocks)

fn outer_block_content_or_raw(md_content: &str) -> String {
	md::outer_block_content_or_raw(md_content)
}

// endregion: --- Rhai Functions
