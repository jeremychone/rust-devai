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
use crate::support::strings::{self, truncate_with_ellipsis};
use crate::Result;
use mlua::{Lua, Table, Value};
use std::borrow::Cow;

pub fn init_module(lua: &Lua, _runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("escape_decode", lua.create_function(escape_decode)?)?;
	table.set("escape_decode_if_needed", lua.create_function(escape_decode_if_needed)?)?;
	table.set("remove_first_line", lua.create_function(remove_first_line)?)?;
	table.set("remove_first_lines", lua.create_function(remove_first_lines)?)?;
	table.set("remove_last_lines", lua.create_function(remove_last_lines)?)?;
	table.set("remove_last_line", lua.create_function(remove_last_line)?)?;
	table.set("truncate", lua.create_function(truncate)?)?;
	table.set(
		"replace_markers",
		lua.create_function(replace_markers_with_default_parkers)?,
	)?;
	table.set(
		"ensure_single_ending_newline",
		lua.create_function(ensure_single_ending_newline)?,
	)?;

	Ok(table)
}

// region:    --- Strings

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
	let new_content = strings::replace_markers(&content, &sections, DEFAULT_MARKERS)?;
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
	Ok(crate::support::strings::ensure_single_ending_newline(content))
}

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

// endregion: --- Strings

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
