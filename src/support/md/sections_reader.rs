use crate::types::{MdHeading, MdSection, ParseResponse};
use crate::{Error, Result};
use simple_fs::SFile;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// This function opens the file and creates the reader, then passes it to `read_md_section`
/// Design to buffer read the file to optimize memory.
/// If ref_section is empty, then, meaning the content before the first heading
pub fn read_file_md_sections(path: impl AsRef<Path>, ref_sections: &[&str]) -> Result<Vec<MdSection>> {
	// Open the file using the given path
	let sfile = SFile::from_path(path)?;
	let file = File::open(sfile)?;

	// Create a buffered reader from the file
	let reader = BufReader::new(file);

	// Call read_md_section with the reader to perform the actual reading
	read_md_sections(reader, ref_sections)
}

#[derive(Debug)]
enum LineData {
	Content(String),
	Heading(MdHeading),
	#[allow(unused)] // does not hurt to keep it for now.
	Blockquote(String),
}

#[derive(Debug)]
enum ActionState {
	NoCapture,
	SkipLineInCapture,
	NewMatchingHeading { ref_idx: usize },
	CaptureLine,
	CloseCurrentSection,
}

/// This function takes a generic reader and reads its content to allow read String readers for example.
/// Note: This separation of concern is mostly for testing as if we have the full content,
///       we will probably do a `.lines()` to get the `&str` and avoid string allocation.
fn read_md_sections<R: Read>(reader: R, ref_headings: &[&str]) -> Result<Vec<MdSection>> {
	// Create a buffered reader to read the content line by line
	let reader = BufReader::new(reader);

	let mut sections: Vec<MdSection> = Vec::new();

	let ref_headings: Vec<(usize, String)> = ref_headings
		.iter()
		.map(|&ref_heading| {
			if ref_heading.is_empty() {
				Ok((0, "".to_string()))
			} else {
				MdHeading::peek_line(ref_heading)
					.ok_or_else(|| {
						Error::custom(format!(
							"read_md_section - ref '{ref_heading}' is not a value markdown heading"
						))
					})
					.map(|(level, name)| (level, name.to_string()))
			}
		})
		.collect::<Result<Vec<_>>>()?;

	// returns the index
	let ref_headings_matches_fn =
		|heading: &MdHeading| ref_headings.iter().position(|(level, name)| heading.matches(*level, name));
	let ref_headings_level_0_fn = || ref_headings.iter().position(|(level, _)| *level == 0);

	let mut current_captured_content: Option<Vec<String>> = None;
	let mut current_captured_heading: Option<MdHeading> = None;
	let mut current_matching_ref: Option<&(usize, String)> = None;
	// This is a flag to capture level 0
	let mut passed_first_heading = false;

	let mut action_state = ActionState::NoCapture;

	// InFunction function to make sure we have the same logic to close section
	fn close_section(
		current_captured_content: &mut Option<Vec<String>>,
		current_matching_ref: &mut Option<&(usize, String)>,
		current_captured_heading: &mut Option<MdHeading>,
		sections: &mut Vec<MdSection>,
	) {
		if let Some(content) = current_captured_content.take() {
			*current_matching_ref = None;
			// TODO: needs to stream the content array to be faster and avoid double allocation
			//       Also we ensure last single new line
			let content = content.join("\n").trim().to_string();
			sections.push(MdSection::new(content, current_captured_heading.take()))
		}
	}

	// let mut line_it = reader.lines();
	// Iterate through each line in the reader and push it into the capture vector
	for line in reader.lines() {
		let line = line?;

		// -- Compute the LineData
		let line_data = match MdHeading::parse_line(line) {
			ParseResponse::Item(heading) => {
				passed_first_heading = true;
				LineData::Heading(heading)
			}
			ParseResponse::Other(line) => {
				if line.starts_with('>') {
					LineData::Blockquote(line)
				} else {
					LineData::Content(line)
				}
			}
		};

		// -- Compute the ActionState
		action_state = match &line_data {
			// -- if the line is a heading,
			// - We need to check if lower level
			LineData::Heading(line_heading) => {
				// if the the current matching ref is 0 and this is a heading
				// we put the matching current_matching_ref = None
				if current_matching_ref.map(|v| v.0 == 0).unwrap_or_default() {
					ActionState::CloseCurrentSection
				} else {
					match current_captured_heading.as_ref() {
						// if we already have a heading, then, it's the end
						Some(captured_heading) => {
							// if we are a level down, we keep capturing
							if line_heading.level() > captured_heading.level() {
								ActionState::CaptureLine
							}
							// TODO: need to add support for CloseAndNewMatchingHeading in there are two adjacent section with saem atmching edding
							// otherwise close the current section
							else {
								ActionState::CloseCurrentSection
							}
						}
						None => {
							if let Some(ref_idx) = ref_headings_matches_fn(line_heading) {
								ActionState::NewMatchingHeading { ref_idx }
							} else {
								action_state
							}
						}
					}
				}
			}
			// Otherwise we change the previous state
			LineData::Content(_) => match action_state {
				ActionState::NoCapture => {
					if !passed_first_heading {
						if let Some(ref_idx) = ref_headings_level_0_fn() {
							ActionState::NewMatchingHeading { ref_idx }
						} else {
							ActionState::NoCapture
						}
					} else {
						ActionState::NoCapture
					}
				}
				ActionState::SkipLineInCapture => ActionState::CaptureLine,
				ActionState::NewMatchingHeading { .. } => ActionState::CaptureLine,
				ActionState::CaptureLine => ActionState::CaptureLine,
				ActionState::CloseCurrentSection => ActionState::NoCapture, // todo: need to access this one.
			},

			LineData::Blockquote(_) => {
				match action_state {
					// Regular no capture if we not in a match section
					ActionState::NoCapture | ActionState::CloseCurrentSection => ActionState::NoCapture,
					// Otherwise, mark it as SkipLineInCapture
					_ => ActionState::SkipLineInCapture,
				}
			}
		};

		// -- Capture data
		match action_state {
			ActionState::NoCapture | ActionState::SkipLineInCapture => (),
			ActionState::CaptureLine => match line_data {
				LineData::Content(line) => current_captured_content.get_or_insert_with(Vec::new).push(line),
				LineData::Heading(line_heading) => current_captured_content
					.get_or_insert_with(Vec::new)
					.push(line_heading.into_content()),
				// For now, we do not capture any blockquote
				LineData::Blockquote(_) => (),
			},
			//
			ActionState::NewMatchingHeading { ref_idx } => {
				close_section(
					&mut current_captured_content,
					&mut current_matching_ref,
					&mut current_captured_heading,
					&mut sections,
				);
				// now start the ref
				match line_data {
					LineData::Heading(line_heading) => {
						let matching_ref =
							ref_headings.get(ref_idx).ok_or("ERROR - No matching ref but should have one")?;
						current_matching_ref = Some(matching_ref);
						current_captured_heading = Some(line_heading);
					}
					// if we are here, it must be a level 0 heading catch
					LineData::Content(line) => {
						let matching_ref =
							ref_headings.get(ref_idx).ok_or("ERROR - No matching ref but should have one")?;
						current_matching_ref = Some(matching_ref);
						current_captured_content.get_or_insert_with(Vec::new).push(line);
					}
					_ => (),
				}
			}
			ActionState::CloseCurrentSection => {
				// ADD MD_SECTION
				close_section(
					&mut current_captured_content,
					&mut current_matching_ref,
					&mut current_captured_heading,
					&mut sections,
				);
			}
		}
	}

	close_section(
		&mut current_captured_content,
		&mut current_matching_ref,
		&mut current_captured_heading,
		&mut sections,
	);

	Ok(sections)
}

// region:    --- Tests

/// This is for test only as for full string, we will probably not have a BufferReader
/// (to avoid new alocation per line.)
#[cfg(test)]
fn read_string_md_sections(content: impl Into<String>, sections: &[&str]) -> Result<Vec<MdSection>> {
	// Use Cursor to wrap the String and provide it as a reader
	let reader = std::io::Cursor::new(content.into());

	// Call read_md_section with the reader to perform the actual reading
	read_md_sections(reader, sections)
}

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, assert_not_contains};

	// region:    --- consts

	const MD_01: &str = r#"
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
	fn test_md_sections_reader_heading_1_root() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = MD_01;
		let fx_headings = &["# Heading 1"];

		// -- Exec
		let MdSection { heading, content } = read_string_md_sections(fx_md, fx_headings)?
			.into_iter()
			.next()
			.ok_or("Should have return result")?;

		// -- Check
		let heading = heading.ok_or("Should have heading")?;
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
	fn test_md_sections_reader_heading_1_a() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = MD_01;
		let fx_headings = &["## sub heading 1-a"];

		// -- Exec
		let MdSection { heading, content } = read_string_md_sections(fx_md, fx_headings)?
			.into_iter()
			.next()
			.ok_or("Should have return result")?;

		// -- Check
		let heading = heading.ok_or("Should have heading")?;
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
	fn test_md_sections_reader_level_0() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = MD_01;
		let fx_heading = &[""];

		// -- Exec
		let MdSection { heading, content } = read_string_md_sections(fx_md, fx_heading)?
			.into_iter()
			.next()
			.ok_or("Should have return result")?;

		// -- Check
		assert!(heading.is_none(), "heading sould be none");
		assert_contains(&content, "Some early text");

		// Should not contain
		assert_not_contains(&content, "# Heading 1");
		assert_not_contains(&content, "content-2");
		assert_not_contains(&content, "heading-1-content");

		Ok(())
	}
}

// endregion: --- Tests
