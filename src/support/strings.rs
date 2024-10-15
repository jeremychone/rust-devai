//! String utils

use std::borrow::Cow;

/// unescape code (sometime chatgpt encode the < and such)
pub fn decode_html_entities(content: &str) -> String {
	html_escape::decode_html_entities(&content).to_string()
}

pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> Cow<str> {
	if s.len() > max_len {
		let truncated = &s[..max_len];
		Cow::from(format!("{}...", truncated))
	} else {
		Cow::from(s)
	}
}

/// Make sure that the text end with one and only one single newline
/// NOT USED FOR NOW - not sure we need this
pub fn _adjust_single_ending_newline(mut text: String) -> String {
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
