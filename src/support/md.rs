use crate::support::adjust_single_ending_newline;

pub fn md_extract_first_rust_block(content: &str) -> Option<String> {
	let mut res = String::new();

	let mut start_capture = false;
	let mut first_code_line = true;

	for line in content.lines() {
		if !start_capture && line.starts_with("```rust") {
			start_capture = true;
			continue;
		}

		if start_capture && line.starts_with("```") {
			break;
		}

		if start_capture {
			if !first_code_line {
				res.push('\n');
				res.push_str(line);
			} else {
				res.push_str(line);
				first_code_line = false
			}
		}
	}

	if res.is_empty() {
		None
	} else {
		// now make sure that the string ends with one and only one newline
		res = adjust_single_ending_newline(res);

		Some(res)
	}
}
