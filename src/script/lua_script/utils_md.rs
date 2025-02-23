//! Defines the `md` module, used in the lua engine.
//!
//! ## Lua Documentation
//! The `md` module exposes functions that process markdown content. Useful for
//! processing LLM responses.
//!
//! ### Functions
//! * `utils.md.extract_blocks(md_content: string, lang?: string) -> Vec<MdBlock>`
//! * `utils.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}) -> Vec<MdBlock>, extruded_content`
//! * `utils.md.extract_meta(md_content) -> Table, String`
//! * `utils.md.outer_block_content_or_raw(md_content: string) -> string`

use crate::Result;
use crate::run::RuntimeContext;
use crate::support::md::{self};
use crate::support::{Extrude, W};
use crate::types::MdBlock;
use mlua::{IntoLua, Lua, LuaSerdeExt, MultiValue, Table, Value};

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	let extract_blocks_fn = lua.create_function(extract_blocks)?;
	let outer_block_content_or_raw_fn = lua.create_function(outer_block_content_or_raw)?;
	let extract_meta_fn = lua.create_function(extract_meta)?;

	table.set("extract_blocks", extract_blocks_fn)?;
	table.set("extract_meta", extract_meta_fn)?;
	table.set("outer_block_content_or_raw", outer_block_content_or_raw_fn)?;

	Ok(table)
}

/// ## Lua Documentation
/// ```lua
/// -- Extract all blocks
/// utils.md.extract_blocks(md_content: string) -> Vec<MdBlock>
/// -- Extract blocks for the language 'lang'
/// utils.md.extract_blocks(md_content: string, lang: string) -> Vec<MdBlock>
/// -- Extract blocks (with or without language, and extrude: content, which the remaining content)
/// utils.md.extract_blocks(md_content: String, {lang: string, extrude: "content"})
/// ```
///
/// Return the list of markdown blocks that match a given lang_name.
fn extract_blocks(lua: &Lua, (md_content, options): (String, Option<Value>)) -> mlua::Result<MultiValue> {
	let (lang, extrude): (Option<String>, Option<Extrude>) = match options {
		// if options is of type string, then, just lang name
		Some(Value::String(string)) => (Some(string.to_string_lossy()), None),
		// if it is a table
		Some(Value::Table(table)) => {
			let lang = table.get::<Option<Value>>("lang")?;
			let lang = lang
				.map(|v| {
					v.to_string()
						.map_err(|_err| crate::Error::custom("md_extract_blocks lang options must be of type string"))
				})
				.transpose()?;

			let extrude = table.get::<Option<Value>>("extrude")?;
			let extrude = extrude
				.map(|extrude| match extrude {
					Value::String(extrude) => {
						if extrude.to_str().unwrap() == "content" {
							Ok(Some(Extrude::Content))
						} else {
							Err(crate::Error::custom(
								"md_extract_blocks extrude must be = to 'content' for now",
							))
						}
					}
					_ => Ok(None),
				})
				.transpose()?
				.flatten();

			(lang, extrude)
		}
		// TODO: Probably need to send error
		_ => (None, None),
	};

	let blocks_it = md::MdBlockIter::new(&md_content, lang.as_deref(), extrude);
	let mut values = MultiValue::new();

	match extrude {
		Some(Extrude::Content) => {
			let (blocks, content) = blocks_it.collect_blocks_and_extruded_content();
			values.push_back(blocks.into_lua(lua)?);
			let content = lua.create_string(&content)?;
			values.push_back(Value::String(content));
		}
		_ => {
			let blocks: Vec<MdBlock> = blocks_it.collect();
			values.push_back(blocks.into_lua(lua)?)
		}
	}

	Ok(values)
}

/// ## Lua Documentation
/// ```lua
/// let meta, remain = utils.md.extract_meta(md_content: string) -> table, string
/// ```
///
/// Extracts the meta blocks, parses/merges their values, and also returns the remaining concatenated content.
///
fn extract_meta(lua: &Lua, md_content: String) -> mlua::Result<MultiValue> {
	let (value, remain) = md::extract_meta(&md_content)?;
	let lua_value = lua.to_value(&value)?;
	let values = MultiValue::from_vec(vec![lua_value, W(remain).into_lua(lua)?]);
	Ok(values)
}

/// ## Lua Documentation
/// ```lua
/// utils.md.outer_block_content_or_raw(md_content: string) -> string
/// ```
///
/// Without fully parsing the markdown, this function will remove the first and last
/// code block (triple back tick), only if the first line is a ` ``` `
///
/// If it does not start with a ` ``` ` raw content will be returned.
///
/// > Note: This is useful in the GenAI context because often LLMs return a top block (e.g., markdown, Rust)
/// >       And while it is better to try to handle this with the prompt, gpt-4o-mini or other models still put in markdown block
fn outer_block_content_or_raw(_lua: &Lua, md_content: String) -> mlua::Result<String> {
	let res = md::outer_block_content_or_raw(&md_content);
	Ok(res.into_owned())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, assert_not_contains, eval_lua, run_reflective_agent, setup_lua};
	use serde_json::Value;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_md_extract_blocks_simple() -> Result<()> {
		// -- Setup & Fixtures
		// NOTE: the [[ ]] for multi line in lua breaks with the ``` for code block, so, reading files.
		let fx_script = r#"
local file = utils.file.load("agent-script/agent-before-all-inputs-gen.aip")
return utils.md.extract_blocks(file.content, {lang = "lua"})
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check
		assert!(res.is_array());
		let blocks = res.as_array().ok_or("Res should be array")?;
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
	async fn test_lua_md_extract_blocks_with_lang_and_extruded_content() -> Result<()> {
		// -- Setup & Fixtures
		// NOTE: the [[ ]] for multi line in lua breaks when line starts with ```, so work around
		let fx_script = r#"
local content = "This is some content\n"
content = content .. "\n```lua\n--some lua \n```\n"
content = content .. "and other block\n\n```rust\n//! some rust block \n```\n"
content = content .. "The end"

local blocks, extruded_content = utils.md.extract_blocks(content, {lang = "lua", extrude = "content"})
return {
		blocks = blocks,
		extruded_content = extruded_content
}
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check Blocks
		let blocks = res.pointer("/blocks").ok_or("Should have blocks")?;
		assert!(blocks.is_array());
		let blocks = blocks.as_array().unwrap();
		assert_eq!(blocks.len(), 1, "Should have found 1 lua blocks");

		// Check first and only blockblock
		let first_block = &blocks[0];
		assert_eq!(first_block.x_get_str("lang")?, "lua");
		assert!(first_block.x_get_str("content")?.contains("some lua"));

		// -- Check Extruded Content
		let content = res.x_get_str("extruded_content")?;
		assert_contains(content, "This is some content");
		assert_contains(content, "and other block");
		assert_contains(content, "```rust");
		assert_contains(content, "```\n");
		assert_contains(content, "The end");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_md_extract_blocks_with_all_lang_and_extruded_content() -> Result<()> {
		// -- Setup & Fixtures
		// NOTE: the [[ ]] for multi line in lua breaks when line starts with ```, so work around
		let fx_script = r#"
local content = "This is some content\n"
content = content .. "\n```lua\n--some lua \n```\n"
content = content .. "and other block\n\n```rust\n//! some rust block \n```\n"
content = content .. "The end"

local blocks, extruded_content = utils.md.extract_blocks(content, {extrude = "content"})
return {
		blocks = blocks,
		extruded_content = extruded_content
}
		"#;

		// -- Exec
		let res = run_reflective_agent(fx_script, None).await?;

		// -- Check Blocks
		let blocks = res.pointer("/blocks").ok_or("Should have blocks")?;
		assert!(blocks.is_array());
		let blocks = blocks.as_array().unwrap();
		assert_eq!(blocks.len(), 2, "Should have found 2 blocks, lua and rust");

		// Check first and only blockblock
		let block = &blocks[0];
		assert_eq!(block.x_get_str("lang")?, "lua");
		assert!(block.x_get_str("content")?.contains("some lua"));
		// Check second block
		let block = &blocks[1];
		assert_eq!(block.x_get_str("lang")?, "rust");
		assert!(block.x_get_str("content")?.contains("some rust"));

		// -- Check Extruded Content
		let content = res.x_get_str("extruded_content")?;
		assert_contains(content, "This is some content");
		assert_contains(content, "and other block");
		assert_not_contains(content, "```lua");
		assert_not_contains(content, "```rust");
		assert_not_contains(content, "```");
		assert_contains(content, "The end");

		Ok(())
	}

	#[test]
	fn test_lua_md_extract_meta() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "md")?;
		let lua_code = r#"
local content = [[
Some content
```toml
#!meta
some = "stuff"
```
some more content
```toml
#!meta
# Another meta block
num = 123
```
And this is the end
]]
local meta, remain = utils.md.extract_meta(content)
return {
   meta   = meta,
	 remain = remain
}
		"#;

		// -- Exec
		let res: Value = eval_lua(&lua, lua_code)?;

		// -- Check meta
		let meta = res.get("meta").ok_or("Should have meta")?;
		assert_eq!(meta.x_get_str("some")?, "stuff");
		assert_eq!(meta.x_get_i64("num")?, 123);

		// -- Check remain
		let remain = res.x_get_str("remain")?;
		assert_contains(remain, "Some content");
		assert_contains(remain, "some more content");
		assert_contains(remain, "And this is the end");
		assert_not_contains(remain, "Another meta block");
		assert_not_contains(remain, "num = 123");
		assert_not_contains(remain, "#!meta");

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
