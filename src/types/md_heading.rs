use derive_more::{Debug, Display};
use mlua::IntoLua;
use std::str::CharIndices;

#[derive(Display, Debug)]
#[display("{content}")]
#[debug("\"{content}\"")]
pub struct MdHeading {
	content: String,
	/// Represents the start and end positions of the name in the raw line
	/// NOTE: Does NOT contain newline at the end.
	name_span: (usize, usize),
	level: usize,
}

/// Getters
impl MdHeading {
	/// Return the full line with the `#`
	/// NOTE: Does not end with newline
	pub fn content(&self) -> &str {
		&self.content
	}

	/// Only the name trimmed
	pub fn name(&self) -> &str {
		&self.content[self.name_span.0..self.name_span.1]
	}

	pub fn level(&self) -> usize {
		self.level
	}

	pub fn matches(&self, level: usize, name: &str) -> bool {
		self.level == level && self.name() == name.trim()
	}
}

pub enum ParseResponse<T> {
	Item(T),
	Other(String),
}

/// Constructors
impl MdHeading {
	pub fn peek_line(line: &str) -> Option<(usize, &str)> {
		let mut chars = line.char_indices();
		let level = MdHeading::parse_level_inner(&mut chars)?;
		let name_span = MdHeading::continue_parse_for_name_span(&mut chars)?;
		Some((level, &line[name_span.0..name_span.1]))
	}

	pub fn parse_line(line: impl Into<String>) -> ParseResponse<MdHeading> {
		let line: String = line.into();
		let mut chars = line.char_indices();

		let Some(level) = MdHeading::parse_level_inner(&mut chars) else {
			return ParseResponse::Other(line);
		};

		let Some(name_span) = MdHeading::continue_parse_for_name_span(&mut chars) else {
			return ParseResponse::Other(line);
		};

		let content = line;

		ParseResponse::Item(MdHeading {
			content,
			name_span,
			level,
		})
	}

	#[cfg(test)]
	fn new(line: impl Into<String>) -> Option<Self> {
		let line: String = line.into();
		let mut chars = line.char_indices();

		// Use helper function to determine heading level and start of content
		let level = MdHeading::parse_level_inner(&mut chars)?;
		let name_span = MdHeading::continue_parse_for_name_span(&mut chars)?;

		Some(MdHeading {
			content: line,
			name_span,
			level,
		})
	}
}

/// Transformers
impl MdHeading {
	pub fn into_content(self) -> String {
		self.content
	}
}

// region:    --- Lua

impl IntoLua for MdHeading {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		let name = self.name().to_string();
		table.set("content", self.content)?;
		table.set("level", self.level)?;
		table.set("name", name)?;

		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua

/// Helper type functions
impl MdHeading {
	/// Helper function to extract heading level and validate format.
	/// Returns None if not a valid heading line (not start with series of `#` and whitespace just after )
	fn parse_level_inner(chars: &mut CharIndices<'_>) -> Option<usize> {
		let mut level = 0;
		// Count the number of leading `#` characters
		for (_, c) in chars.by_ref() {
			// if leading whitespace, it's invalid
			if level == 0 && c.is_whitespace() {
				return None;
			} else if c == '#' {
				level += 1;
			}
			// if no whitespace after the last #, then not valid heading
			else if !c.is_whitespace() {
				return None;
			} else {
				break;
			}
		}

		if level == 0 || level > 6 {
			return None; // Must have at least one `#` and at most 6 `#`
		}

		Some(level)
	}

	/// Internal function to continue to parse for the nmae
	/// the chars interator must have been gone through the `parse_level_inner` first
	fn continue_parse_for_name_span(chars: &mut CharIndices<'_>) -> Option<(usize, usize)> {
		// Find start and end positions of the heading name
		let mut start_pos = None;
		let mut end_pos = None;

		// This will start at the first whitespace after the last `#`
		for (i, c) in chars {
			if c == '\n' {
				break;
			}
			if start_pos.is_none() && !c.is_whitespace() {
				start_pos = Some(i); // First non-whitespace character after `#`
			}
			if start_pos.is_some() {
				end_pos = Some(i + 1); // Update end position to the last non-whitespace character
			}
		}
		Some((start_pos?, end_pos?))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_md_heading_simple_ok() {
		let headings = vec![
			("# Heading One", 1, "Heading One"),
			("## Heading Two", 2, "Heading Two"),
			("### Heading Three", 3, "Heading Three"),
			("##### Heading Five", 5, "Heading Five"),
			("###### Heading Six", 6, "Heading Six"),
		];

		for (line, expected_level, expected_name) in headings {
			let heading = MdHeading::new(line);

			assert!(heading.is_some(), "Failed to parse valid heading: '{}'", line);

			let heading = heading.unwrap();

			assert_eq!(heading.level, expected_level, "Incorrect level for heading: '{}'", line);
			assert_eq!(heading.name(), expected_name, "Incorrect name for heading: '{}'", line);
		}
	}

	#[test]
	fn test_md_heading_simple_none() {
		let invalid_headings = vec![
			" Heading without hash",                           // No leading '#'
			"    #### Heading with leading space",             // Should fail because leading spaces
			"####### Too many hashes",                         // More than 6 '#'
			"#NoSpaceAfterHash",                               // No space after '#'
			"",                                                // Empty line
			"    ",                                            // Only whitespace
			"###",                                             // Only '#' without content
			"   ##No space after leading whitespace and hash", // No space after `##` following whitespace
		];

		for line in invalid_headings {
			let heading = MdHeading::new(line.to_string());
			assert!(
				heading.is_none(),
				"Parsed an invalid heading that should be None: '{}'",
				line
			);
		}
	}
}
