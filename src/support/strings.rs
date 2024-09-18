//! String utils

/// unescape code (sometime chatgpt encode the < and such)
pub fn code_escape_decode(content: &str) -> String {
	html_escape::decode_html_entities(&content).to_string()
}

pub fn adjust_single_ending_newline(mut s: String) -> String {
	if s.is_empty() {
		s.push('\n'); // If the string is empty, just add a newline
		return s;
	}

	// Note: Some, perhaps unnecessary, optimization to avoid traversing the whole string or doing unnecessary allocation.
	let chars = s.chars().rev(); // Create an iterator that traverses the string backwards

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
		0 => s.push('\n'),                              // No trailing newlines, add one
		1 => (),                                        // Exactly one newline, do nothing
		_ => s.truncate(s.len() - (newline_count - 1)), // More than one newline, remove extra
	}

	s
}
