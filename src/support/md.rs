pub fn md_extract_first_rust_block(content: &str) -> Option<String> {
	let mut res = String::new();

	let mut start_capture = false;
	for line in content.lines() {
		if !start_capture && line.starts_with("```rust") {
			start_capture = true;
			continue;
		}
		if start_capture && line.starts_with("```") {
			return Some(res);
		}
		if start_capture {
			res.push_str(line);
			res.push('\n');
		}
	}

	None
}
