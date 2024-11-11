//! String utils

use crate::{Error, Result};
use aho_corasick::AhoCorasick;
use std::borrow::Cow;

pub fn truncate_with_ellipsis<'a>(s: &'a str, max_len: usize, ellipsis: &str) -> Cow<'a, str> {
	if s.len() > max_len {
		let truncated = &s[..max_len];
		if ellipsis.is_empty() {
			// no allocation needed
			Cow::from(truncated)
		} else {
			Cow::from(format!("{truncated}{ellipsis}"))
		}
	} else {
		Cow::from(s)
	}
}

#[allow(unused)]
pub fn replace_all(content: &str, patterns: &[&str], values: &[&str]) -> Result<String> {
	let ac = AhoCorasick::new(patterns).map_err(|err| Error::cc("replace_all fail because patterns", err))?;

	let res = ac.replace_all_bytes(content.as_bytes(), values);
	let new_content =
		String::from_utf8(res).map_err(|err| Error::cc("replace_all fail because result is not utf8", err))?;

	Ok(new_content)
}

/// Make sure that the text end with one and only one single newline
/// Useful for code sanitization
pub fn ensure_single_ending_newline(mut text: String) -> String {
	if text.is_empty() {
		text.push('\n'); // If the string is empty, just add a newline
		return text;
	}

	// Note: Some, perhaps unnecessary, optimization to avoid traversing the whole string or doing unnecessary allocation.
	let chars = text.chars().rev(); // Create an iterator that traverses the string backwards

	// Count the number of trailing newlines
	let mut newline_count = 0;
	for ch in chars {
		if ch == '\n' {
			newline_count += 1;
		} else {
			break;
		}
	}

	match newline_count {
		0 => text.push('\n'),                                 // No trailing newlines, add one
		1 => (),                                              // Exactly one newline, do nothing
		_ => text.truncate(text.len() - (newline_count - 1)), // More than one newline, remove extra
	}

	text
}
