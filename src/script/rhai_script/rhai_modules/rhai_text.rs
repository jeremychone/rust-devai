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

use crate::support::html::decode_html_entities;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};

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

	// ensure_single_ending_newline
	FuncRegistration::new("ensure_single_ending_newline")
		.in_global_namespace()
		.set_into_module(&mut module, ensure_single_ending_newline);

	module
}

// region:    --- Strings

/// ## RHAI Documentation
/// ```rhai
/// text::ensure_single_ending_newline(content: string) -> string
/// ```
///
/// Ensures that `content` ends with a single newline character.
/// If `content` is empty, it returns a newline character.
///
/// This function is useful for code sanitization.
fn ensure_single_ending_newline(content: &str) -> String {
	// Note: This string::ensure... is optimized to
	crate::support::strings::ensure_single_ending_newline(content.to_string())
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
