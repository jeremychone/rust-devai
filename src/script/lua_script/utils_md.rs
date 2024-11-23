//! Defines the `md` module, used in the lua engine.
//!
//! ## Lua Documentation
//! The `md` module exposes functions that process markdown content. Useful for
//! processing LLM responses.
//!
//! ### Functions
//! * `md.extract_blocks(md_content: string, lang_name?: string) -> Vec<MdBlock>`
//! * `md.outer_block_content_or_raw(md_content: string) -> string`

use crate::run::RuntimeContext;
use crate::support::md;
use crate::types::MdBlock;
use crate::Result;
use mlua::{IntoLua, Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let extract_blocks_with_lang_fn = lua.create_function(extract_blocks_with_lang)?;
	let outer_block_content_or_raw_fn = lua.create_function(outer_block_content_or_raw)?;

	table.set("extract_blocks", extract_blocks_with_lang_fn)?;
	table.set("outer_block_content_or_raw", outer_block_content_or_raw_fn)?;

	Ok(table)
}

/// ## Lua Documentation
/// ```lua
/// md.extract_blocks(md_content: string, lang_name: string) -> Vec<MdBlock>
/// ```
///
/// Return the list of markdown blocks that match a given lang_name.
fn extract_blocks_with_lang(lua: &Lua, (md_content, lang_name): (String, Option<String>)) -> mlua::Result<Value> {
	let blocks: Vec<MdBlock> = md::MdBlocks::new(&md_content, lang_name.as_deref()).collect();
	blocks.into_lua(lua)
}

/// ## Lua Documentation
/// ```lua
/// md.outer_block_content_or_raw(md_content: string) -> string
/// ```
///
/// Without fully parsing the markdown, this function attempts to extract the content from the first set of triple backticks
/// to the last set of triple backticks.
/// If no starting or ending triple backticks are found, it will return the raw content.
///
/// > Note: This is useful in the GenAI context because often LLMs return a top block (e.g., markdown, Rust)
/// >       which might have other ` ``` ` in the middle but should be interpreted as nested.
/// >       (GenAI does not seem to recognize the use of 6 backticks for top-level blocks)
fn outer_block_content_or_raw(_lua: &Lua, md_content: String) -> mlua::Result<String> {
	let res = md::outer_block_content_or_raw(&md_content);
	Ok(res)
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_md_extract_blocks() -> Result<()> {
		// -- Setup & Fixtures
		// NOTE: the [[ ]] for multi line in lua breaks with the ``` for code block, so, reading files.
		let fx_script = r#"
local file = utils.file.load("agent-script/agent-before-all-inputs-gen.devai")
return utils.md.extract_blocks(file.content, "lua")
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check
		assert!(res.is_array());
		let blocks = res.as_array().unwrap();
		assert_eq!(blocks.len(), 4, "Should have found 4 lua blocks");

		// Check first block
		let first_block = &blocks[0];
		assert_eq!(first_block.x_get_str("lang")?, "lua");
		assert!(first_block.x_get_str("content")?.contains("before_all_response"));

		// Check second block
		let second_block = &blocks[1];
		assert_eq!(second_block.x_get_str("lang")?, "lua");
		assert!(second_block.x_get_str("content")?.contains("Data with input"));

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_md_outer_block_content_or_raw() -> Result<()> {
		// -- Setup & Fixtures
		// NOTE: Here we put the ``` out of the multiline [[ ]]
		let fx_script = r#"        
local content = "```" .. [[rust
fn main() {
    // Some nested blocks
    let example = ```typescript
    const x = 42;
    ```;
    println!("Hello!");
}
]] .. "```"

return utils.md.outer_block_content_or_raw(content)
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check
		let content = res.as_str().unwrap();
		assert!(content.contains("fn main()"));
		assert!(content.contains("const x = 42"));
		assert!(!content.contains("```rust")); // Should not contain the outer markers

		// Test with raw content (no blocks)
		let fx_script_raw = r#"
local content = [[Just some plain
text without any code blocks]]

return utils.md.outer_block_content_or_raw(content)
		"#;

		let res_raw = run_reflective_agent(fx_script_raw, None).await?;
		let content_raw = res_raw.as_str().unwrap();
		assert_eq!(content_raw, "Just some plain\ntext without any code blocks");

		Ok(())
	}
}
// endregion: --- Tests
