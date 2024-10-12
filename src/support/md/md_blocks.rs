use crate::types::MdBlock;

/// Represents an iterator over Markdown code blocks with optional language filtering.
pub struct MdBlocks<'a> {
	lines: std::str::Lines<'a>,
	lang_filter: Option<&'a str>,
}

impl<'a> MdBlocks<'a> {
	/// Creates a new MdBlocks iterator from the given content.
	///
	/// # Arguments
	///
	/// * `content` - The Markdown content to iterate over.
	/// * `lang_filter` - An optional language filter:
	///     - `None`: Any code block is returned.
	///     - `Some(s)`: Only code blocks with a matching language are returned.
	///       - If `s` is an empty string, only code blocks without a specified language are returned.
	pub fn new(content: &'a str, lang_filter: Option<&'a str>) -> Self {
		MdBlocks {
			lines: content.lines(),
			lang_filter,
		}
	}

	/// Retrieves the next `MdBlock` that matches the language filter, if any.
	///
	/// This function searches for the next code block in the Markdown content that satisfies the language filter criteria.
	/// It skips any code blocks that do not match the filter and continues searching until a matching block is found or the content ends.
	fn next_block(&mut self) -> Option<MdBlock> {
		// if the line is inside a block and contains the lang, can be empty string
		let mut in_block: Option<&str> = None;
		let mut captured_content: Option<String> = None;

		for line in self.lines.by_ref() {
			// -- Check if new block and capture lang
			if line.starts_with("```") {
				// We are entering a new block
				if in_block.is_none() {
					// extract the lang
					let lang = line.trim_start_matches("```").trim();
					in_block = Some(lang);

					// determine if content needs to be captured
					captured_content = match self.lang_filter {
						Some(filter) => {
							if filter == lang {
								Some(String::new())
							} else {
								None
							}
						}
						None => Some(String::new()),
					};
				}
				// We are exciting a new block
				else {
					if let Some(content) = captured_content {
						return Some(MdBlock {
							lang: Some(in_block.unwrap_or_default().to_string()),
							content,
						});
					}

					in_block = None;
					captured_content = None;
				}

				continue;
			} // if line.starts_with("```")

			// -- Capture the content
			if let Some(content) = &mut captured_content {
				content.push_str(line);
				content.push('\n');
			}
		}

		// No more blocks found
		None
	}
}

/// Implementing Iterator for MdBlocks to yield `MdBlock` directly.
impl<'a> Iterator for MdBlocks<'a> {
	type Item = MdBlock;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_block()
	}
}

// region:    --- Tests

#[path = "../../_tests/tests_support_md_blocks.rs"]
#[cfg(test)]
mod tests;

// endregion: --- Tests
