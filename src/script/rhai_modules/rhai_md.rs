//! Defines the `md` module, used in the rhai engine. 
//! 
//! ## RHAI Documentation
//! The `md` module exposes functions that process markdown content. Useful for
//! processing LLM responses.
//! 
//! ### Functions
//! * `extract_blocks(md_content: string) -> Vec<MdBlock>`
//! * `extract_blocks(md_content: string, lang_name: string) -> Vec<MdBlock>`

use crate::support::md;
use crate::types::MdBlock;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, Module};

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("extract_blocks")
		.in_global_namespace()
		.set_into_module(&mut module, extract_blocks);

	FuncRegistration::new("extract_blocks")
		.in_global_namespace()
		.set_into_module(&mut module, extract_blocks_with_name);

	module
}

// region:    --- Rhai Functions

/// ## RHAI Documentation
/// ```rhai
/// extract_blocks(md_content: string) -> Vec<MdBlock>
/// ```
/// 
/// Parses the markdown provided by `md_content` and extracts each code block, 
/// returning a list of blocks.
fn extract_blocks(md_content: &str) -> RhaiResult {
	let blocks = md::extract_blocks(md_content, None);
	let blocks: Vec<Dynamic> = blocks.into_iter().map(MdBlock::into_dynamic).collect();
	Ok(blocks.into())
}

/// ## RHAI Documentation
/// ```rhai
/// extract_blocks(md_content: &str, lang_name: &str) -> Vec<MdBlock>
/// ```
/// 
/// Parses the markdown provided by `md_content` and extracts each code block,
/// returning only the blocks with a 
/// [language identifier](https://docs.github.com/en/get-started/writing-on-github/working-with-advanced-formatting/creating-and-highlighting-code-blocks#syntax-highlighting)
/// that matches `lang_name`.
fn extract_blocks_with_name(md_content: &str, lang_name: &str) -> RhaiResult {
	let blocks = md::extract_blocks(md_content, Some(lang_name));
	let blocks: Vec<Dynamic> = blocks.into_iter().map(MdBlock::into_dynamic).collect();
	Ok(blocks.into())
}

// endregion: --- Rhai Functions
