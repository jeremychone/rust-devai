//! Iterator over text blocks delimited by lines that start with a given prefix.
//!
//! This implementation concatenates all consecutive lines starting with the prefix into a block.
//! Lines that do not start with the prefix are collected as extruded content.

use crate::support::Extrude;

/// Options for configuring the LineBlockIter.
pub struct LineBlockIterOptions<'b> {
	pub starts_with: &'b str,
	pub extrude: Option<Extrude>,
}

pub struct LineBlockIter<'a> {
	starts_with: &'a str,
	extrude: Option<Extrude>,
	/// The content lines iterator.
	lines: std::str::Lines<'a>,
	/// Collected extruded content (lines outside of any block).
	extruded_content: Vec<&'a str>,
}

impl<'a> LineBlockIter<'a> {
	/// Creates a new TextBlock iterator from the given content and options.
	///
	/// # Arguments
	///
	/// * `content` - The text content to iterate over.
	/// * `options` - The options containing the marker prefix and extrude configuration.
	pub fn new(content: &'a str, options: LineBlockIterOptions<'a>) -> Self {
		LineBlockIter {
			starts_with: options.starts_with,
			extrude: options.extrude,
			lines: content.lines(),
			extruded_content: Vec::new(),
		}
	}

	/// Retrieves the next block as a concatenated string of all consecutive lines
	/// that start with the given prefix.
	///
	/// A block starts when a line beginning with the prefix is found and continues
	/// until a non-prefixed line is encountered. The non-prefixed line is added to the
	/// extruded content.
	fn next_block(&mut self) -> Option<String> {
		let mut current_block = String::new();
		let mut in_block = false;
		let extrude_content = matches!(&self.extrude, Some(Extrude::Content));

		for line in self.lines.by_ref() {
			if line.starts_with(self.starts_with) {
				in_block = true;
				current_block.push_str(line);
				current_block.push('\n');
			} else {
				if extrude_content {
					self.extruded_content.push(line);
					self.extruded_content.push("\n");
				}

				if in_block {
					// End the current block when a non-prefix line is encountered.
					return Some(current_block);
				}
				// Not in a block; just continue scanning.
			}
		}

		// If the text ended while collecting a block, return it.
		if in_block {
			return Some(current_block);
		}
		None
	}

	/// Consumes the iterator to collect all text blocks and the extruded content.
	///
	/// Returns a tuple where:
	/// - The first element is a vector of blocks (each block is a string).
	/// - The second element is a string of all extruded content.
	pub fn collect_blocks_and_extruded_content(mut self) -> (Vec<String>, String) {
		let mut blocks: Vec<String> = Vec::new();

		for block in self.by_ref() {
			blocks.push(block);
		}

		let extruded_content = self.extruded_content.join("");
		(blocks, extruded_content)
	}
}

impl<'a> Iterator for LineBlockIter<'a> {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_block()
	}
}

// region: --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_support_text_line_block_iter_simple() {
		// -- Setup & Fixtures
		let content = "\
> one
> two
line1
> three
Some extruded line";

		// -- Exec
		let mut iter = LineBlockIter::new(
			content,
			LineBlockIterOptions {
				starts_with: ">",
				extrude: None,
			},
		);

		// -- Check
		// check Blocks
		let block = iter.next().expect("First Block should be returned");
		assert_eq!(block, "> one\n> two\n");

		let block = iter.next().expect("Second Block should be returned");
		assert_eq!(block, "> three\n");

		assert!(iter.next().is_none());

		// check Content
		assert_eq!(iter.extruded_content.len(), 0, "extruded content vec should be 0");
	}

	#[test]
	fn test_support_text_line_block_iter_extrude_content_simple() {
		// -- Setup & Fixtures
		let content = "\
> one
> two
line1
> three
Some extruded line";

		// -- Exec: Collect both blocks and extruded content.
		let (blocks, extruded_content) = LineBlockIter::new(
			content,
			LineBlockIterOptions {
				starts_with: ">",
				extrude: Some(Extrude::Content),
			},
		)
		.collect_blocks_and_extruded_content();

		// -- Check Blocks
		assert_eq!(blocks.len(), 2);
		assert_eq!(blocks[0], "> one\n> two\n");
		assert_eq!(blocks[1], "> three\n");

		// -- Check that the extruded content is as expected.
		assert_eq!(extruded_content, "line1\nSome extruded line\n");
	}
}

// endregion: --- Tests
