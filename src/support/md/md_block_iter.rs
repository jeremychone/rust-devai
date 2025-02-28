use crate::support::Extrude;
use crate::support::md::InBlockState;
use crate::types::MdBlock; // new import to support 3/6 ticks

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

/// Constructor and main iterator function
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
		// Use InBlockState to manage code block boundaries (3 or 6 ticks)
		let mut block_state = InBlockState::Out;
		let mut current_lang: Option<String> = None;
		let mut captured_content: Option<Vec<&'a str>> = None;
		let extrude_content = matches!(self.extrude, Some(Extrude::Content));

		for line in self.lines.by_ref() {
			let previous_state = block_state;
			block_state = block_state.compute_new(line);

			// Detect entering a new code block
			if previous_state.is_out() && !block_state.is_out() {
				let lang = match block_state {
					InBlockState::In4 => line.strip_prefix("````").unwrap_or(line).trim(),
					InBlockState::In3 => line.strip_prefix("```").unwrap_or(line).trim(),
					_ => line.trim(), // unreachable
				};
				// Store the language for later use when constructing MdBlock.
				current_lang = Some(lang.to_string());
				captured_content = match self.lang_filter {
					Some(filter) => {
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
				continue;
			}

			// Detect exiting a code block
			if !previous_state.is_out() && block_state.is_out() {
				if let Some(content) = captured_content.take() {
					let joined = content.join("");
					let block = MdBlock {
						lang: current_lang.clone(),
						content: joined,
					};
					return Some(block);
				} else if extrude_content {
					self.extruded_content.push(line);
					self.extruded_content.push("\n");
				}
				current_lang = None;
				continue;
			}

			// When inside a code block, capture the content if applicable
			if !block_state.is_out() {
				if let Some(ref mut cap) = captured_content {
					cap.push(line);
					cap.push("\n");
				} else if extrude_content {
					self.extruded_content.push(line);
					self.extruded_content.push("\n");
				}
			} else {
				// Outside a block, extrude content if necessary.
				if extrude_content {
					self.extruded_content.push(line);
					self.extruded_content.push("\n");
				}
			}
		}

		None
	}
}

impl MdBlockIter<'_> {
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
impl Iterator for MdBlockIter<'_> {
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
