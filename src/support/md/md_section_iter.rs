use crate::support::md::InBlockState;
use crate::types::{MdHeading, MdSection, ParseResponse};
use crate::{Error, Result};
use std::borrow::{BorrowMut, Cow};
use std::io::BufRead;
use std::path::Path;
use std::{fs, io, str};

// region:    --- CowLines

/// Enum to represent Cow Lines iterator
pub enum CowLines<'a> {
	StrLines(str::Lines<'a>),
	FileLines(io::Lines<io::BufReader<fs::File>>),
}

impl<'a> Iterator for CowLines<'a> {
	type Item = Cow<'a, str>;

	fn next(&mut self) -> Option<Self::Item> {
		match self {
			CowLines::StrLines(lines) => lines.next().map(Cow::Borrowed),
			CowLines::FileLines(lines) => lines.next().map(|line| Cow::Owned(line.unwrap())),
		}
	}
}

impl<'a> CowLines<'a> {
	fn from_str(content: &'a str) -> Self {
		CowLines::StrLines(content.lines())
	}

	fn from_path(path: impl AsRef<Path>) -> Result<Self> {
		let file = std::fs::File::open(path)?;
		let reader = io::BufReader::new(file);
		Ok(CowLines::FileLines(reader.lines()))
	}
}

impl<'a> CowLines<'a> {
	/// Joins a `Vec<Cow<str>>` into a single `String` efficiently.
	pub fn join(&mut self, separator: &str) -> String {
		let parts: Vec<Cow<str>> = self.collect();
		join_cows(parts, separator)
	}
}

fn join_cows(parts: Vec<Cow<str>>, separator: &str) -> String {
	let total_len: usize = parts.iter().map(|s| s.len()).sum();
	let mut result = String::with_capacity(total_len + separator.len() * (parts.len().saturating_sub(1)));
	for (i, part) in parts.iter().enumerate() {
		if i > 0 {
			result.push_str(separator);
		}
		result.push_str(part);
	}
	result
}

// endregion: --- CowLines

/// The Section filter pattern.
/// Right now, supports heading only which is a good start and efficient since no need to look ahead.
#[derive(Debug)]
struct SectionPattern {
	heading_level: usize,
	heading_name: String,
}

impl SectionPattern {
	fn new(heading_level: usize, heading_name: &str) -> Self {
		Self {
			heading_level,
			heading_name: heading_name.trim().to_string(),
		}
	}

	fn matches(&self, level: usize, name: &str) -> bool {
		self.heading_level == level && self.heading_name == name
	}

	fn matches_heading(&self, heading: &MdHeading) -> bool {
		self.matches(heading.level(), heading.name())
	}
}

// (level, name) (name is the trimmed version after the last "#" of the heading)
// (0, "") means root (no heading)

/// Represents an iterator over Markdown sections with multiple section filters.
pub struct MdSectionIter<'a> {
	// -- Iterator data
	lines: CowLines<'a>,
	/// A filter is a list of SectionPatterns
	filter: Vec<SectionPattern>,

	// -- iterator managed states
	passed_first_heading: bool,
	action_state: ActionState,
	last_heading: Option<MdHeading>,
}

impl<'a> MdSectionIter<'a> {
	/// Creates a new MdSection iterator from the given content source.
	fn new(source: CowLines<'a>, heading_patterns: Option<&[&str]>) -> Result<Self> {
		let section_filter = Self::resolve_heading_patterns(heading_patterns).unwrap();

		Ok(MdSectionIter {
			lines: source,
			filter: section_filter,
			passed_first_heading: false,
			action_state: ActionState::NoCapture,
			last_heading: None,
		})
	}

	pub fn from_path(path: impl AsRef<Path>, heading_patterns: Option<&[&str]>) -> Result<Self> {
		let lines = CowLines::from_path(path)?;
		Self::new(lines, heading_patterns)
	}

	pub fn from_str(content: &'a str, heading_patterns: Option<&[&str]>) -> Result<Self> {
		let lines = CowLines::from_str(content);
		Self::new(lines, heading_patterns)
	}

	fn resolve_heading_patterns(heading_patterns: Option<&[&str]>) -> Result<Vec<SectionPattern>> {
		let Some(heading_patterns) = heading_patterns else {
			return Ok(Vec::new());
		};

		let ref_headings: Vec<SectionPattern> = heading_patterns
			.iter()
			.map(|&ref_heading| {
				if ref_heading.is_empty() {
					Ok(SectionPattern::new(0, ""))
				} else {
					MdHeading::peek_line(ref_heading)
						.ok_or_else(|| {
							Error::custom(format!(
								"read_md_section - ref '{ref_heading}' is not a valid markdown heading"
							))
						})
						.map(|(level, name)| SectionPattern::new(level, name))
				}
			})
			.collect::<Result<Vec<_>>>()?;

		Ok(ref_headings)
	}

	/// Retrieves the next `MdSection` that matches the section filters, if any.
	fn next_section(&mut self) -> Option<MdSection> {
		// -- Helper function to close and return the section
		fn close_section(
			current_captured_content: &mut Option<Vec<Cow<str>>>,
			current_captured_heading: &mut Option<MdHeading>,
		) -> Option<MdSection> {
			current_captured_content.take().map(|content| {
				let content = join_cows(content, "\n");
				MdSection::new(content, current_captured_heading.take())
			})
		}

		// -- Helper closures to look in the filters
		let filter_matches_fn =
			|heading: &MdHeading| self.filter.iter().position(|pattern| pattern.matches_heading(heading));
		let filter_has_level_0_fn = || self.filter.iter().position(|pattern| pattern.heading_level == 0);

		// -- function states
		// if filter list is empty, then, we capture all sections
		let all_sections = self.filter.is_empty();
		let mut current_captured_content: Option<Vec<Cow<'a, str>>> = None;
		let mut current_captured_heading: Option<MdHeading> = self.last_heading.take();
		let mut current_matching_ref: Option<&SectionPattern> = None;
		// to make sure we
		let mut block_state = InBlockState::Out;
		// This is a flag to capture level 0

		// -- Capture the content and return when found
		for line in self.lines.borrow_mut() {
			block_state = block_state.compute_new(line.as_ref());
			let is_inside_code_block = !block_state.is_out();

			/// -- Capture the LineData
			let line_data = {
				if is_inside_code_block {
					if line.starts_with('>') {
						LineData::Blockquote(line)
					} else {
						LineData::Content(line)
					}
				} else {
					//
					match MdHeading::peek_line(line.as_ref()) {
						Some((level, name)) => {
							self.passed_first_heading = true;
							// TODO: need to return LineData::Heading should have the (level, name, line: Cow...)
							match MdHeading::parse_line(line.to_string()) {
								ParseResponse::Item(heading) => LineData::Heading(heading),
								// TODO: Here we should hever have Other, as the peek_line return Some.
								//       But just in case, fall back to other
								ParseResponse::Other(_) => LineData::Content(line),
							}
						}
						None => {
							if line.starts_with('>') {
								LineData::Blockquote(line)
							} else {
								LineData::Content(line)
							}
						}
					}
				}
			};

			// -- Compute the action
			self.action_state = match &line_data {
				// -- We are on a Heading
				LineData::Heading(line_heading) => {
					// if we capture all section, this is a new section
					if all_sections {
						ActionState::NewHeadingForAllSections
					}
					// If the current matching ref is 0 and this is a heading
					// we set the matching current_matching_ref = None
					else if current_matching_ref
						.map(|pattern| pattern.heading_level == 0)
						.unwrap_or_default()
					{
						ActionState::CloseCurrentSection
					} else {
						match current_captured_heading.as_ref() {
							// If were capturing the heading, then only capture if lower
							Some(captured_heading) => {
								if line_heading.level() > captured_heading.level() {
									ActionState::CaptureLine
								} else {
									ActionState::CloseCurrentSection
								}
							}
							// If we were not in a capture heading
							None => {
								if let Some(ref_idx) = filter_matches_fn(line_heading) {
									ActionState::NewMatchingHeading { ref_idx }
								} else {
									// same state as before (for now clone, but ok because statck/size. Can be optimized later.)
									self.action_state.clone()
								}
							}
						}
					}
				}
				// -- If we are on a content line
				LineData::Content(_) => match self.action_state {
					ActionState::NoCapture => {
						if !self.passed_first_heading {
							if let Some(ref_idx) = filter_has_level_0_fn() {
								ActionState::NewMatchingHeading { ref_idx }
							} else {
								// Now, we chech if all_sections
								if all_sections {
									ActionState::CaptureLine
								} else {
									ActionState::NoCapture
								}
							}
						} else {
							// Now, we chech if all_sections
							if all_sections {
								ActionState::CaptureLine
							} else {
								ActionState::NoCapture
							}
						}
					}
					ActionState::SkipLineInCapture => ActionState::CaptureLine,
					ActionState::NewMatchingHeading { .. } => ActionState::CaptureLine,
					ActionState::NewHeadingForAllSections => ActionState::CaptureLine,
					ActionState::CaptureLine => ActionState::CaptureLine,
					ActionState::CloseCurrentSection => ActionState::NoCapture,
				},
				LineData::Blockquote(_) => match self.action_state {
					ActionState::NoCapture | ActionState::CloseCurrentSection => ActionState::NoCapture,
					_ => ActionState::SkipLineInCapture,
				},
			};

			// -- Execute the action
			match self.action_state {
				ActionState::NoCapture | ActionState::SkipLineInCapture => (),
				ActionState::CaptureLine => match line_data {
					LineData::Content(line) => current_captured_content.get_or_insert_with(Vec::new).push(line),
					LineData::Heading(line_heading) => current_captured_content
						.get_or_insert_with(Vec::new)
						.push(line_heading.into_content().into()),
					LineData::Blockquote(_) => (),
				},
				ActionState::NewHeadingForAllSections => match line_data {
					LineData::Heading(line_heading) => {
						/// if we are in a NewHeadingForAllSections and already capturin something, we close current
						if current_captured_heading.is_some() || current_captured_content.is_some() {
							self.last_heading = Some(line_heading);
							return close_section(&mut current_captured_content, &mut current_captured_heading);
						}
						// if we start a new capture, just set it
						else {
							current_captured_heading = Some(line_heading);
						}
					}
					LineData::Content(line) => {
						current_captured_content.get_or_insert_with(Vec::new).push(line);
					}
					_ => (),
				},
				ActionState::NewMatchingHeading { ref_idx } => {
					match line_data {
						LineData::Heading(line_heading) => {
							// TODO: Probalby need to have the same logic as above if we start something and during capture

							let Some(matching_ref) = self.filter.get(ref_idx) else {
								// NOTE: Given the logic, this should not happen, but do not panic just return None.
								return None;
							};
							current_matching_ref = Some(matching_ref);
							current_captured_heading = Some(line_heading);
						}
						// if we are here, it must be a level 0 heading catch
						LineData::Content(line) => {
							let Some(matching_ref) = self.filter.get(ref_idx) else {
								// NOTE: Given the logic, this should not happen, but do not panic just return None.
								return None;
							};
							current_matching_ref = Some(matching_ref);
							current_captured_content.get_or_insert_with(Vec::new).push(line);
						}
						_ => (),
					}
				}
				ActionState::CloseCurrentSection => {
					return close_section(&mut current_captured_content, &mut current_captured_heading);
				}
			}
		}

		close_section(&mut current_captured_content, &mut current_captured_heading)
	}
}

impl<'a> Iterator for MdSectionIter<'a> {
	type Item = MdSection;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_section()
	}
}

#[derive(Debug)]
enum LineData<'a> {
	Content(Cow<'a, str>),
	Heading(MdHeading),
	#[allow(unused)]
	Blockquote(Cow<'a, str>),
}

#[derive(Debug, Clone)]
enum ActionState {
	NoCapture,
	SkipLineInCapture,
	/// WE are in all sections match, and this is a new heading
	NewHeadingForAllSections,
	/// we have a new matching eadhing for a ref_idx in the SectionPattern array
	NewMatchingHeading {
		ref_idx: usize,
	},
	CaptureLine,
	CloseCurrentSection,
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, assert_not_contains};

	// region:    --- consts

	const FX_MD_01: &str = r#"
Some early text

# Heading 1

> Some heading-1-blockquote

Some heading-1-content

And some more heading-1-other-content

## sub heading 1-a

> Some heading-1-a-blockquote

Some heading-1-a-content

Some heading-1-a-other-content 

# Heading 2

Some other content-2

```
# Some block content for content-2
```

# Heading three
		"#;

	// endregion: --- consts

	#[test]
	fn test_md_section_iter_no_filter() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &[""];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, None)?;
		let mut sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 5, "Should have only 5 sections match");
		// check first section
		let first = sections.first().ok_or("SHould have first section")?;
		assert!(first.heading().is_none(), "First section heading should be none");
		assert_contains(first.content(), "Some early text");
		// Check last
		let last = sections.last().ok_or("Should have last section")?;
		let last_heading = last.heading().ok_or("Last section heading should be some")?;
		assert_eq!(last_heading.name(), "Heading three");
		assert_eq!(last_heading.level(), 1);
		assert_contains(last.content().trim(), "");

		// // extract first
		// let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;

		// // // check heading
		// assert!(heading.is_none(), "heading should be none");
		// // // Should not content
		// assert_contains(&content, "Some early text");
		// assert_not_contains(&content, "# Heading 1");
		// assert_not_contains(&content, "content-2");
		// assert_not_contains(&content, "heading-1-content");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_reader_heading_1_root() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &["# Heading 1"];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, Some(fx_headings))?;
		let mut sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 1, "Should have only one match");
		// extract first
		let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;
		// check heading
		let heading = heading.ok_or("Should have a heading")?;
		assert_eq!(heading.line(), fx_headings[0]);
		assert_eq!(heading.level(), 1);
		// Should contain
		assert_contains(&content, "heading-1-content");
		assert_contains(&content, "Some heading-1-a-content");
		assert_contains(&content, "Some heading-1-a-other-content");
		// Should not contain
		assert_not_contains(&content, fx_headings[0]);
		assert_not_contains(&content, "heading-1-blockquote");
		assert_not_contains(&content, "heading-1-a-blockquote");
		assert_not_contains(&content, "Some early text");
		assert_not_contains(&content, "content-2");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_reader_heading_1_a() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &["## sub heading 1-a"];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, Some(fx_headings))?;
		let mut sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 1, "Should have only one match");
		// extract first
		let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;
		// check heading
		let heading = heading.ok_or("Should have a heading")?;
		assert_eq!(heading.line(), fx_headings[0]);
		assert_eq!(heading.level(), 2);
		// Should contain
		assert_contains(&content, "Some heading-1-a-content");
		assert_contains(&content, "Some heading-1-a-other-content");
		// Should not contain
		assert_not_contains(&content, fx_headings[0]);
		assert_not_contains(&content, "Some early text");
		assert_not_contains(&content, "content-2");
		assert_not_contains(&content, "heading-1-content");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_reader_level_0() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &[""];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, Some(fx_headings))?;
		let mut sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 1, "Should have only one match");
		// extract first
		let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;

		// // check heading
		assert!(heading.is_none(), "heading should be none");
		// // Should not content
		assert_contains(&content, "Some early text");
		assert_not_contains(&content, "# Heading 1");
		assert_not_contains(&content, "content-2");
		assert_not_contains(&content, "heading-1-content");

		Ok(())
	}
}

// endregion: --- Tests
