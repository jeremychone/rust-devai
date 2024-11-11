//! Defines the `text` module, used in the rhai engine
//!
//! ---
//!
//! ## RHAI documentation
//! This module exposes functions that process text.
//!
//! ### Functions
//! * `text::escape_decode(content: string) -> string`
//! * `text::escape_decode_if_needed(content: string) -> string`
//! * `text::remove_first_line(content: string) -> string`
//! * `text::remove_first_lines(content: string, n: int) -> string`
//! * `text::remove_last_line(content: string) -> string`
//! * `text::remove_last_lines(content: string, n: int) -> string`
//! * `text::replace_markers(content: string, new_sections: array) -> string`

use crate::script::rhai_script::rhai_modules::DEFAULT_MARKERS;
use crate::support::html::decode_html_entities;
use crate::support::strings::{self, truncate_with_ellipsis};
use crate::Error;
use rhai::plugin::RhaiResult;
use rhai::{Dynamic, FuncRegistration, ImmutableString, Module};
use std::borrow::Cow;
use std::ops::Deref;

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	// Register the functions to the module

	FuncRegistration::new("escape_decode")
		.in_global_namespace()
		.set_into_module(&mut module, escape_decode);

	FuncRegistration::new("escape_decode_if_needed")
		.in_global_namespace()
		.set_into_module(&mut module, escape_decode_if_needed);

	FuncRegistration::new("remove_first_line")
		.in_global_namespace()
		.set_into_module(&mut module, remove_first_line);

	FuncRegistration::new("remove_first_lines")
		.in_global_namespace()
		.set_into_module(&mut module, remove_first_lines);

	FuncRegistration::new("remove_last_lines")
		.in_global_namespace()
		.set_into_module(&mut module, remove_last_lines);

	FuncRegistration::new("remove_last_line")
		.in_global_namespace()
		.set_into_module(&mut module, remove_last_line);

	FuncRegistration::new("remove_last_line")
		.in_global_namespace()
		.set_into_module(&mut module, remove_last_line);

	FuncRegistration::new("remove_last_line")
		.in_global_namespace()
		.set_into_module(&mut module, remove_last_line);

	FuncRegistration::new("truncate")
		.in_global_namespace()
		.set_into_module(&mut module, |content: &str, max_len: usize| {
			truncate(content, max_len, "")
		});

	FuncRegistration::new("truncate")
		.in_global_namespace()
		.set_into_module(&mut module, |content: &str, max_len: usize, ellipsis: &str| {
			truncate(content, max_len, ellipsis)
		});

	FuncRegistration::new("replace_markers")
		.in_global_namespace()
		.set_into_module(&mut module, replace_markers_with_default_parkers);

	// ensure_single_ending_newline
	FuncRegistration::new("ensure_single_ending_newline")
		.in_global_namespace()
		.set_into_module(&mut module, ensure_single_ending_newline);

	module
}

// region:    --- Strings

fn replace_markers_with_default_parkers(content: &str, new_sections: Vec<Dynamic>) -> RhaiResult {
	// TODO: Should try to optimize this to use the static string and get the &str (Vec<&str> does not work)
	const NEW_SECTION_ERROR: &str =
		"A new section item is not of type string or does not contain a .content of type string";
	let new_sections = new_sections
		.iter()
		.map(|x| {
			let im_string = if let Ok(map) = x.as_map_ref() {
				let map = map.deref();
				if let Some(content) = map.get("content") {
					// to stuff with content
					content
						.as_immutable_string_ref()
						.ok()
						.map(|v| v.as_str().to_string())
						.ok_or_else(|| Error::custom(NEW_SECTION_ERROR))
				} else {
					Err(Error::custom(NEW_SECTION_ERROR))
				}
			} else {
				x.as_immutable_string_ref()
					.ok()
					.map(|v| v.as_str().to_string())
					.ok_or_else(|| Error::custom(NEW_SECTION_ERROR))
			};
			im_string
		})
		.collect::<crate::Result<Vec<String>>>()?;

	let new_sections: Vec<&str> = new_sections.iter().map(|x| x.as_str()).collect();

	let new_content = strings::replace_markers(content, &new_sections, DEFAULT_MARKERS)?;
	Ok(new_content.into())
}

/// ## RHAI Documentation
/// ```rhai
/// text::truncate(content: string, max_len: int) -> string
/// // can include a ellipsis which will be added if the text is bigger than max_len
/// text::truncate(content: string, max_len: int, ellipsis: string) -> string
/// ```
///
/// Returns `content` truncated to a maximum length of `max_len`.
/// If the content exceeds `max_len`, it appends the optional `ellipsis` string to indicate truncation.
/// If `ellipsis` is not provided, no additional characters are added after truncation.
///
/// This function is useful for limiting the length of text output while preserving meaningful context.
fn truncate(content: &str, max_len: usize, ellipsis: &str) -> Dynamic {
	match truncate_with_ellipsis(content, max_len, ellipsis) {
		Cow::Borrowed(txt) => txt.into(),
		Cow::Owned(txt) => txt.into(),
	}
}

/// ## RHAI Documentation
/// ```rhai
/// text::ensure_single_ending_newline(content: string) -> string
/// ```
///
/// Ensures that `content` ends with a single newline character.
/// If `content` is empty, it returns a newline character.
///
/// This function is useful for code normalization.
fn ensure_single_ending_newline(content: &str) -> RhaiResult {
	// Note: Might want to optimize if possible
	let s = crate::support::strings::ensure_single_ending_newline(content.to_string());
	Ok(s.into())
}

///  ## RHAI Documentation
/// ```rhai
/// text::remove_first_line(content: string) -> string
/// ```
///
/// Returns `content` with the first line removed.
fn remove_first_line(content: &str) -> &str {
	remove_first_lines(content, 1)
}

///  ## RHAI Documentation
/// ```rhai
/// text::remove_first_lines(content: string, n: int) -> string
/// ```
///
/// Returns `content` with the first `n` lines removed.
fn remove_first_lines(content: &str, num_of_lines: usize) -> &str {
	let mut start_idx = 0;
	let mut newline_count = 0;

	// Iterate over the bytes of the string to find the `num_of_lines`-th newline character
	for (i, c) in content.char_indices() {
		if c == '\n' {
			newline_count += 1;
			if newline_count == num_of_lines {
				start_idx = i + 1; // The start of the remaining content
				break;
			}
		}
	}

	// If num_of_lines is greater than the total number of lines, return an empty string
	if newline_count < num_of_lines {
		return "";
	}

	// Return the remaining content from `start_idx` to the end of the string
	&content[start_idx..]
}

///  ## RHAI Documentation
/// ```rhai
/// text::remove_last_line(content: string) -> string
/// ```
///
/// Returns `content` with the last line removed.
fn remove_last_line(content: &str) -> &str {
	remove_last_lines(content, 1)
}

///  ## RHAI Documentation
/// ```rhai
/// text::remove_last_lines(content: string, n: int) -> string
/// ```
///
/// Returns `content` with the last `n` lines removed.
fn remove_last_lines(content: &str, num_of_lines: usize) -> &str {
	let mut end_idx = content.len(); // Start with the end of the string
	let mut newline_count = 0;

	// Iterate over the characters of the string in reverse
	for (i, c) in content.char_indices().rev() {
		if c == '\n' {
			newline_count += 1;
			if newline_count == num_of_lines {
				end_idx = i; // Set end index to the beginning of the last `n` lines
				break;
			}
		}
	}

	// If num_of_lines is greater than the total number of lines, return an empty string
	if newline_count < num_of_lines {
		return "";
	}

	// Return the content from the start up to `end_idx`
	&content[..end_idx]
}

// endregion: --- Strings

// region:    --- Escape Fns

/// Only escape if needed. right now, the test only test `&lt;`
///
/// ## RHAI Documentation
/// ```rhai
/// text::escape_decode(content: string) -> string
/// ```
///
/// Some LLMs HTML-encode their responses. This function returns `content`
/// after selectively decoding certain HTML tags.
///
/// Right now, the only tag that gets decoded is `&lt;`.
fn escape_decode_if_needed(content: &str) -> RhaiResult {
	if !content.contains("&lt;") {
		Ok(content.into())
	} else {
		escape_decode(content)
	}
}

// html-escape
/// ## RHAI Documentation
/// ```rhai
/// text::escape_decode(content: string) -> string
/// ```
///
/// Some LLMs HTML-encode their responses. This function returns `content`,
/// HTML-decoded.
fn escape_decode(content: &str) -> RhaiResult {
	let decoded = decode_html_entities(content);
	Ok(decoded.into())
}

// endregion: --- Escape Fns
