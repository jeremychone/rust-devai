use crate::support::adjust_single_ending_newline;
use crate::types::MdBlock;
use std::mem;

pub fn extract_blocks(content: &str, with_lang_name: Option<&str>) -> Vec<MdBlock> {
	let mut blocks = Vec::new();
	let mut current_block = String::new();
	let mut current_lang: Option<String> = None;

	let mut inside_block = false;
	let mut start_capture = false;
	let mut first_code_line = true;
	let mut matching_lang = false;

	// TODO: Add support for `````` the six ticks scheme
	for line in content.lines() {
		// Will be inside at first ``` of the pair, and the closing one will be false
		if line.starts_with("```") {
			inside_block = !inside_block;
		}

		// if we are not in a block pair
		if inside_block && line.starts_with("```") {
			let lang_name = line.trim_start_matches("```").trim();
			current_lang = Some(lang_name.to_string());

			// If we have a language name specified, check if the current block matches
			if let Some(with_lang_name) = with_lang_name {
				if with_lang_name == lang_name {
					start_capture = true;
					matching_lang = true;
					continue;
				}
			} else {
				// If no language name is specified, capture any code block
				start_capture = true;
				matching_lang = true;
				continue;
			}
		}

		// -- if second block tick pair
		if !inside_block && line.starts_with("```") {
			if matching_lang {
				// take and clear the current_block
				let content = mem::take(&mut current_block);
				let content = adjust_single_ending_newline(content);
				// create bloxk
				let block = MdBlock::new(current_lang.take(), content);
				blocks.push(block);
			}
			// Reset flags for next block
			current_block.clear();
			current_lang = None;
			start_capture = false;
			first_code_line = true;
			matching_lang = false;
			continue;
		}

		if inside_block && start_capture {
			if !first_code_line {
				current_block.push('\n');
			}
			current_block.push_str(line);
			first_code_line = false;
		}
	}

	blocks
}

// region:    --- Tests

#[path = "../../_tests/tests_support_md.rs"]
#[cfg(test)]
mod tests;

// endregion: --- Tests
