//! Defines the `text` module, used in the lua engine
//!
//! ---
//!
//! ## Lua documentation
//! This module exposes functions that process text.
//!
//! ### Functions
//! * `utils.text.escape_decode(content: string) -> string`
//! * `utils.text.escape_decode_if_needed(content: string) -> string`
//! * `utils.text.remove_first_line(content: string) -> string`
//! * `utils.text.remove_first_lines(content: string, n: int) -> string`
//! * `utils.text.remove_last_line(content: string) -> string`
//! * `utils.text.remove_last_lines(content: string, n: int) -> string`
//! * `utils.text.truncate(content: string, max_len: int) -> string`
//! * `utils.text.truncate(content: string, max_len: int, ellipsis: string) -> string`
//! * `utils.text.replace_markers(content: string, new_sections: array) -> string`
//! * `utils.text.ensure_single_ending_newline(content: string) -> string`

use crate::run::RuntimeContext;
use crate::script::lua_script::helpers::to_vec_of_strings;
use crate::script::lua_script::DEFAULT_MARKERS;
use crate::support::html::decode_html_entities;
use crate::support::text::{self, truncate_with_ellipsis, EnsureOptions};
use crate::Result;
use mlua::{FromLua, Lua, LuaSerdeExt, MultiValue, String as LuaString, Table, Value};
use std::borrow::Cow;

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("escape_decode", lua.create_function(escape_decode)?)?;
	table.set("escape_decode_if_needed", lua.create_function(escape_decode_if_needed)?)?;
	table.set("split_first", lua.create_function(split_first)?)?;
	table.set("remove_first_line", lua.create_function(remove_first_line)?)?;
	table.set("remove_first_lines", lua.create_function(remove_first_lines)?)?;
	table.set("remove_last_lines", lua.create_function(remove_last_lines)?)?;
	table.set("remove_last_line", lua.create_function(remove_last_line)?)?;
	table.set("trim", lua.create_function(trim)?)?;
	table.set("trim_start", lua.create_function(trim_start)?)?;
	table.set("trim_end", lua.create_function(trim_end)?)?;
	table.set("truncate", lua.create_function(truncate)?)?;
	table.set(
		"replace_markers",
		lua.create_function(replace_markers_with_default_parkers)?,
	)?;
	table.set("ensure", lua.create_function(ensure)?)?;
	table.set(
		"ensure_single_ending_newline",
		lua.create_function(ensure_single_ending_newline)?,
	)?;

	Ok(table)
}

// region:    --- ensure

impl FromLua for EnsureOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		let table = value.as_table().ok_or_else(|| {
			mlua::Error::runtime(
				"Ensure argument needs to be a table with the format {start = string, end = string} (both optional",
			)
		})?;

		//
		let prefix = table.get::<String>("prefix").ok();
		let suffix = table.get::<String>("suffix").ok();

		for (key, _value) in table.pairs::<Value, Value>().flatten() {
			if let Some(key) = key.as_str() {
				if key != "prefix" && key != "suffix" {
					let msg = format!("Ensure argument contains invalid table property `{key}`. Can only contain `prefix` and/or `suffix`");
					return Err(mlua::Error::RuntimeError(msg));
				}
			}
		}

		//
		Ok(EnsureOptions { prefix, suffix })
	}
}

/// ## Lua Documentation
/// ```lua
/// utils.text.ensure(content: string, {prefix? = string, suffix? = string}) -- string
/// ```
///
/// Ensure the content start and/or end with the text given in the second argument dictionary.
///
/// This function is useful for code normalization.
fn ensure(lua: &Lua, (content, inst): (String, Value)) -> mlua::Result<String> {
	let inst = EnsureOptions::from_lua(inst, lua)?;
	let res = crate::support::text::ensure(&content, inst);
	let res = res.to_string();
	Ok(res)
}

/// ## Lua Documentation
/// ```lua
/// text.ensure_single_ending_newline(content: string) -> string
/// ```
///
/// Ensures that `content` ends with a single newline character.
/// If `content` is empty, it returns a newline character.
///
/// This function is useful for code normalization.
fn ensure_single_ending_newline(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(crate::support::text::ensure_single_ending_newline(content))
}

// endregion: --- ensure

// region:    --- Transform

/// ## Lua Documentation
/// ```lua
/// text.replace_markers(content: string, new_sections: array) -> string
/// ```
///
/// Replaces markers in `content` with corresponding sections from `new_sections`.
/// Each section in `new_sections` can be a string or a map containing a `.content` string.
fn replace_markers_with_default_parkers(_lua: &Lua, (content, new_sections): (String, Value)) -> mlua::Result<String> {
	let sections = to_vec_of_strings(new_sections, "new_sections")?;
	let sections: Vec<&str> = sections.iter().map(|s| s.as_str()).collect();
	let new_content = text::replace_markers(&content, &sections, DEFAULT_MARKERS)?;
	Ok(new_content)
}

/// ## Lua Documentation
/// ```lua
/// text.truncate(content: string, max_len: int, ellipsis?: string) -> string
/// ```
///
/// Returns `content` truncated to a maximum length of `max_len`.
/// If the content exceeds `max_len`, it appends the optional `ellipsis` string to indicate truncation.
/// If `ellipsis` is not provided, no additional characters are added after truncation.
///
/// This function is useful for limiting the length of text output while preserving meaningful context.
fn truncate(_lua: &Lua, (content, max_len, ellipsis): (String, usize, Option<String>)) -> mlua::Result<String> {
	let ellipsis = ellipsis.unwrap_or_default();
	match truncate_with_ellipsis(&content, max_len, &ellipsis) {
		Cow::Borrowed(txt) => Ok(txt.to_string()),
		Cow::Owned(txt) => Ok(txt),
	}
}

// endregion: --- Transform

// region:    --- Split

/// ## Luas Documentaiton
/// ```lua
/// local content = "some first content\n===\nsecond content"
/// local first, second = utils.text.split_first(content,"===")
/// -- first  = "some first content\n"
/// -- second = "\nsecond content"
/// -- NOTE: When no match, second is nil.
/// --       If match, but nothing after, second is ""
/// ```
/// NOTE: For optimization, this will use LuaString to avoid converting Lua String to Rust String and back
fn split_first(lua: &Lua, (content, sep): (LuaString, LuaString)) -> mlua::Result<MultiValue> {
	// Convert LuaStrings to Rust strings
	let content_str = content.to_str()?;
	let sep_str = sep.to_str()?;

	// Find the first occurrence of the separator
	if let Some(index) = content_str.find(&*sep_str) {
		// Split the content into two parts
		let first_part = &content_str[..index];
		let second_part = &content_str[index + sep_str.len()..];

		// Convert parts back to Lua strings and return as MultiValue
		Ok(MultiValue::from_vec(vec![
			Value::String(lua.create_string(first_part)?),
			Value::String(lua.create_string(second_part)?),
		]))
	} else {
		// Return the content as the first value and nil as the second
		Ok(MultiValue::from_vec(vec![Value::String(content), Value::Nil]))
	}
}

// endregion: --- Split

// region:    --- Trim

fn trim(lua: &Lua, content: String) -> mlua::Result<Value> {
	lua.to_value(content.trim())
}

fn trim_end(lua: &Lua, content: String) -> mlua::Result<Value> {
	lua.to_value(content.trim_end())
}

fn trim_start(lua: &Lua, content: String) -> mlua::Result<Value> {
	lua.to_value(content.trim_start())
}

// endregion: --- Trim

// region:    --- Remove

///  ## Lua Documentation
/// ```lua
/// text.remove_first_line(content: string) -> string
/// ```
///
/// Returns `content` with the first line removed.
fn remove_first_line(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(remove_first_lines_impl(&content, 1).to_string())
}

///  ## Lua Documentation
/// ```lua
/// text.remove_first_lines(content: string, n: int) -> string
/// ```
///
/// Returns `content` with the first `n` lines removed.
fn remove_first_lines(_lua: &Lua, (content, num_of_lines): (String, i64)) -> mlua::Result<String> {
	Ok(remove_first_lines_impl(&content, num_of_lines as usize).to_string())
}

fn remove_first_lines_impl(content: &str, num_of_lines: usize) -> &str {
	let mut start_idx = 0;
	let mut newline_count = 0;

	for (i, c) in content.char_indices() {
		if c == '\n' {
			newline_count += 1;
			if newline_count == num_of_lines {
				start_idx = i + 1;
				break;
			}
		}
	}

	if newline_count < num_of_lines {
		return "";
	}

	&content[start_idx..]
}

///  ## Lua Documentation
/// ```lua
/// text.remove_last_line(content: string) -> string
/// ```
///
/// Returns `content` with the last line removed.
fn remove_last_line(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(remove_last_lines_impl(&content, 1).to_string())
}

///  ## Lua Documentation
/// ```lua
/// text.remove_last_lines(content: string, n: int) -> string
/// ```
///
/// Returns `content` with the last `n` lines removed.
fn remove_last_lines(_lua: &Lua, (content, num_of_lines): (String, i64)) -> mlua::Result<String> {
	Ok(remove_last_lines_impl(&content, num_of_lines as usize).to_string())
}

fn remove_last_lines_impl(content: &str, num_of_lines: usize) -> &str {
	let mut end_idx = content.len();
	let mut newline_count = 0;

	for (i, c) in content.char_indices().rev() {
		if c == '\n' {
			newline_count += 1;
			if newline_count == num_of_lines {
				end_idx = i;
				break;
			}
		}
	}

	if newline_count < num_of_lines {
		return "";
	}

	&content[..end_idx]
}

// endregion: --- Remove

// region:    --- Escape Fns

/// ## Lua Documentation
/// ```lua
/// text.escape_decode_if_needed(content: string) -> string
/// ```
///
/// Only escape if needed. Right now, the test only tests `&lt;`.
///
/// Some LLMs HTML-encode their responses. This function returns `content`
/// after selectively decoding certain HTML tags.
///
/// Right now, the only tag that gets decoded is `&lt;`.
fn escape_decode_if_needed(_lua: &Lua, content: String) -> mlua::Result<String> {
	if !content.contains("&lt;") {
		Ok(content)
	} else {
		escape_decode(_lua, content)
	}
}

/// ## Lua Documentation
/// ```lua
/// text.escape_decode(content: string) -> string
/// ```
///
/// Some LLMs HTML-encode their responses. This function returns `content`,
/// HTML-decoded.
fn escape_decode(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(decode_html_entities(&content))
}

// endregion: --- Escape Fns

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::run_reflective_agent;

	#[tokio::test]
	async fn test_lua_text_split_first_ok() -> Result<()> {
		// -- Fixtures
		// (content, separator, (first, second))
		let data = [
			// with matching
			(
				"some first content\n===\nsecond content",
				"===",
				("some first content\n", Some("\nsecond content")),
			),
			// no matching
			("some first content\n", "===", ("some first content\n", None)),
			// no matching
			("some first content\n===", "===", ("some first content\n", Some(""))),
		];

		for (content, sep, expected) in data {
			// -- Exec
			// Note 1: Here the content is in debug mode, so the \n does not get "rendered" causing a wrong script.
			// Note 2: Here we get the multi-value result from split_first, but return a table, since the run_command_agent
			//         underneath requires returning a single value (which is the correct approach).
			let script = format!(
				r#"
			local first, second = utils.text.split_first({content:?}, "{sep}")
			return {{first, second}}
			"#
			);
			let res = run_reflective_agent(&script, None).await?;

			// -- Check
			let values = res.as_array().ok_or("Should have returned an array")?;

			// check fist
			let first = values
				.first()
				.ok_or("Should always have at least a first return")?
				.as_str()
				.ok_or("First should be string")?;
			assert_eq!(expected.0, first);

			// check second
			let second = values.get(1); // might be none, when no sep matches, json array is one item (it's ok)
			if let Some(exp_second) = expected.1 {
				let second = second.ok_or("Should have second")?;
				assert_eq!(exp_second, second)
			} else {
				assert!(second.is_none(), "Second should not have been none");
			}
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_ensure_ok() -> Result<()> {
		// -- Fixtures
		let data = [
			(
				"some- ! -path",
				r#"{prefix = "./", suffix = ".md"}"#,
				"./some- ! -path.md",
			),
			("some- ! -path", r#"{suffix = ".md"}"#, "some- ! -path.md"),
			(" ~ some- ! -path", r#"{prefix = " ~ "}"#, " ~ some- ! -path"),
			("~ some- ! -path", r#"{prefix = " ~ "}"#, " ~ ~ some- ! -path"),
		];

		for (content, arg, expected) in data {
			// -- exec
			let script = format!("return utils.text.ensure(\"{content}\", {arg})");
			let res = run_reflective_agent(&script, None).await?;

			// -- Check
			assert_eq!(res, expected);
		}

		Ok(())
	}
}

// endregion: --- Tests
