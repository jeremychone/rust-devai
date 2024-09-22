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

fn extract_blocks(md_content: &str) -> RhaiResult {
	let blocks = md::extract_blocks(md_content, None);
	let blocks: Vec<Dynamic> = blocks.into_iter().map(MdBlock::into_dynamic).collect();
	Ok(blocks.into())
}

fn extract_blocks_with_name(md_content: &str, lang_name: &str) -> RhaiResult {
	let blocks = md::extract_blocks(md_content, Some(lang_name));
	let blocks: Vec<Dynamic> = blocks.into_iter().map(MdBlock::into_dynamic).collect();
	Ok(blocks.into())
}

// endregion: --- Rhai Functions
