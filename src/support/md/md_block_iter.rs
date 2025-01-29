use crate::types::MdBlock;

/// Represents an iterator over Markdown code blocks with optional language filtering.
pub struct MdBlockIter<'a> {
	// -- The iterator options
	lang_filter: Option<&'a str>,
	extrude: Option<Extrude>,
	// -- The states
	/// The content lines iterator
	lines: std::str::Lines<'a>,
	/// The eventual extrude content
	extruded_content: Vec<&'a str>,
}

/// The type of "extrude" to be performed
/// - `Content`: Concatenate all lines into one string
/// - `Segments` (NOT SUPPORTED YET): Have a vector of strings for Before, In Between, and After
#[derive(Debug, Clone, Copy)]
pub enum Extrude {
	Content,
	// Segments, // TODO
}

/// Constructor and main iterator fuction
impl<'a> MdBlockIter<'a> {
	/// Creates a new MdBlock iterator from the given content.
	///
	/// # Arguments
	///
	/// * `content` - The Markdown content to iterate over.
	/// * `lang_filter` - An optional language filter:
	///     - `None`: Any code block is returned.
	///     - `Some(s)`: Only code blocks with a matching language are returned.
	///       - If `s` is an empty string, only code blocks without a specified language are returned.
	pub fn new(content: &'a str, lang_filter: Option<&'a str>, extrude: Option<Extrude>) -> Self {
		MdBlockIter {
			lines: content.lines(),
			lang_filter,
			extrude,
			extruded_content: Vec::new(),
		}
	}

	/// Retrieves the next `MdBlock` that matches the language filter, if any.
	///
	/// This function searches for the next code block in the Markdown content that satisfies the language filter criteria.
	/// It skips any code blocks that do not match the filter and continues searching until a matching block is found or the content ends.
	fn next_block(&mut self) -> Option<MdBlock> {
		// If the line is inside a block and contains the language, it can be an empty string
		let mut in_block: Option<&str> = None;
		let mut captured_content: Option<Vec<&str>> = None;

		let extrude_content = matches!(self.extrude, Some(Extrude::Content));

		for line in self.lines.by_ref() {
			// -- Check if new block and capture language
			if line.starts_with("```") {
				// -- We are entering a new block
				if in_block.is_none() {
					// Extract the language
					let lang = line.trim_start_matches("```").trim();
					in_block = Some(lang);

					// Determine if content needs to be captured
					captured_content = match self.lang_filter {
						Some(filter) => {
							// -- match the filter, so we start capturing the block
							if filter == lang {
								Some(Vec::new())
							} else {
								if extrude_content {
									self.extruded_content.push(line);
									self.extruded_content.push("\n");
								}
								None
							}
						}
						None => Some(Vec::new()),
					};
				}
				// -- We are exiting a block
				else {
					if let Some(content) = captured_content {
						let content = content.join("");
						return Some(MdBlock {
							lang: Some(in_block.unwrap_or_default().to_string()),
							content,
						});
					} else if extrude_content {
						self.extruded_content.push(line);
						self.extruded_content.push("\n");
					}

					in_block = None;
					captured_content = None;
				}

				continue;
			} // if line.starts_with("```")

			// -- Capture the content
			if let Some(content) = &mut captured_content {
				content.push(line);
				content.push("\n");
			}
			// -- If no capture, but extrude_content, then, we extrude it
			else if extrude_content {
				self.extruded_content.push(line);
				self.extruded_content.push("\n");
			}
		}

		// No more blocks found
		None
	}
}

impl<'a> MdBlockIter<'a> {
	pub fn collect_blocks_and_extruded_content(mut self) -> (Vec<MdBlock>, String) {
		let mut blocks: Vec<MdBlock> = Vec::new();

		for block in self.by_ref() {
			blocks.push(block);
		}

		// Now that iteration is done, extract extruded_content
		let extruded_content = self.extruded_content.join("");

		(blocks, extruded_content)
	}
}

/// Implementing Iterator for MdBlock to yield `MdBlock` directly.
impl<'a> Iterator for MdBlockIter<'a> {
	type Item = MdBlock;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_block()
	}
}

// region:    --- Tests

#[path = "../../_tests/tests_support_md_block_iter.rs"]
#[cfg(test)]
mod tests;

// endregion: --- Tests
