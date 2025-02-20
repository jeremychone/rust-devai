//! String utils

use crate::{Error, Result};
use aho_corasick::AhoCorasick;
use derive_more::derive::Display;
use num_format::ToFormattedString;
use std::borrow::Cow;
use std::time::Duration;

pub fn format_num(num: i64) -> String {
	num.to_formatted_string(&num_format::Locale::en)
}

pub fn format_duration(duration: Duration) -> String {
	let duration = Duration::from_millis(duration.as_millis() as u64);
	humantime::format_duration(duration).to_string()
}

// region:    --- Ensure

pub struct EnsureOptions {
	pub prefix: Option<String>,
	pub suffix: Option<String>,
}

pub fn ensure(s: &str, ensure_inst: EnsureOptions) -> Cow<str> {
	let mut parts: Vec<&str> = Vec::new();

	// Add start prefix if needed
	if let Some(start) = ensure_inst.prefix.as_deref() {
		if !s.starts_with(start) {
			parts.push(start);
		}
	}

	// Always include the main string
	parts.push(s);

	// Add end suffix if needed
	if let Some(end) = ensure_inst.suffix.as_deref() {
		if !s.ends_with(end) {
			parts.push(end);
		}
	}

	// If no changes were made, return the original string as borrowed
	if parts.len() == 1 {
		Cow::Borrowed(s)
	} else {
		Cow::Owned(parts.concat()) // Join parts into a single owned string
	}
}
// endregion: --- Ensure

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

// region:    --- Replace

pub fn replace_markers(content: &str, sections: &[&str], marker_pair: &(&str, &str)) -> Result<String> {
	let lines = content.lines();
	let mut section_iter = sections.iter();
	let mut new_content: Vec<&str> = Vec::new();

	let (marker_start, marker_end) = marker_pair;

	#[derive(Display)]
	enum State {
		StartMakerLine,
		InSection,
		EndMarkerLine,
		OutSection,
	}
	let mut state = State::OutSection;

	for line in lines {
		// -- compute next state
		state = if line.contains(marker_start) {
			if matches!(state, State::StartMakerLine | State::InSection) {
				return Err(format!(
					"replace_markers - Cannot have start marker {marker_start} when previous section not closed with {marker_end}"
				)
				.into());
			}
			State::StartMakerLine
		} else if line.contains(marker_end) {
			if matches!(state, State::OutSection) {
				return Err(format!(
					"replace_markers - Cannot have close marker {marker_end} when not having open with a {marker_start}"
				)
				.into());
			}
			State::EndMarkerLine
		} else {
			// compute from prevous state
			// TODO: probably need to do some check as well
			match state {
				State::StartMakerLine => State::InSection,
				State::InSection => State::InSection,
				State::EndMarkerLine => State::OutSection,
				State::OutSection => State::OutSection,
			}
		};

		// -- add to new_content
		match state {
			State::StartMakerLine => (),
			State::InSection => (),
			State::EndMarkerLine => {
				let section = section_iter.next().ok_or("replace_markers - Not enough matching sections")?;
				new_content.push(section);
			}
			State::OutSection => new_content.push(line),
		}
	}

	// make sure to add a new empty line
	let original_end_with_newline = content.as_bytes().last().map(|&b| b == b'\n').unwrap_or_default();
	if original_end_with_newline {
		new_content.push(""); // to have the last empty line on join("\n")
	}

	Ok(new_content.join("\n"))
}

#[allow(unused)]
pub fn replace_all(content: &str, patterns: &[&str], values: &[&str]) -> Result<String> {
	let ac = AhoCorasick::new(patterns).map_err(|err| Error::cc("replace_all fail because patterns", err))?;

	let res = ac.replace_all_bytes(content.as_bytes(), values);
	let new_content =
		String::from_utf8(res).map_err(|err| Error::cc("replace_all fail because result is not utf8", err))?;

	Ok(new_content)
}

// endregion: --- Replace

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, assert_not_contains};

	#[test]
	fn test_support_text_replace_markers_simple() -> Result<()> {
		// -- Setup & Fixtures
		let markers = &("<<START>>", "<<END>>");
		let content = r#"
	This is some content-01
	// <<START>>
	with some instruction for markers. inst-01
	// <<END>>
	and some more content-02
<<START>>
	Another set of instructions here. inst-02
<<END>>	

And more content-03
"#;
		let sections = &["SECTION-01", "// SECTION 02"];

		// -- Exec
		let new_content = replace_markers(content, sections, markers)?;

		// -- Check
		assert_contains(&new_content, "content-01");
		assert_contains(&new_content, "content-02");
		assert_contains(&new_content, "content-03\n");
		assert_contains(&new_content, "SECTION-01");
		assert_contains(&new_content, "// SECTION 02");
		assert_not_contains(&new_content, "<<START>>");
		assert_not_contains(&new_content, "<<END>>");
		assert_not_contains(&new_content, "inst-01");
		assert_not_contains(&new_content, "inst-02");

		Ok(())
	}

	#[test]
	fn test_support_text_markers_no_closing() -> Result<()> {
		// -- Setup & Fixtures
		let markers = &("<<START>>", "<<END>>");
		let content = r#"
	This is some content-01
	// <<START>>
	with some instruction for markers. inst-01
<<START>>
	Another set of instructions here. inst-02
<<END>>	

And more content-03
"#;
		let sections = &["SECTION-01", "// SECTION 02"];

		// -- Exec
		let res = replace_markers(content, sections, markers);

		// -- Check
		if res.is_ok() {
			return Err("Should have fail replace_markers because wrong content".into());
		}

		Ok(())
	}
}

// endregion: --- Tests
