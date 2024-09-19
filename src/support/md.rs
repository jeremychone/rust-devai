use crate::support::adjust_single_ending_newline;

pub fn extract_blocks(content: &str, lang_name: Option<&str>) -> Vec<String> {
	let mut blocks = Vec::new();
	let mut current_block = String::new();

	let mut start_capture = false;
	let mut first_code_line = true;
	let mut matching_lang = false;

	let lang_prefix = lang_name.map(|lang| format!("```{}", lang));

	for line in content.lines() {
		if !start_capture && line.starts_with("```") {
			// If we have a language name specified, check if the current block matches
			if let Some(ref prefix) = lang_prefix {
				if line.starts_with(prefix) {
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

		if start_capture && line.starts_with("```") {
			if matching_lang {
				// End the capture and store the block
				blocks.push(adjust_single_ending_newline(current_block.clone()));
			}
			// Reset flags for next block
			current_block.clear();
			start_capture = false;
			first_code_line = true;
			matching_lang = false;
			continue;
		}

		if start_capture {
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

#[cfg(test)]
mod tests {
	// type Error = Box<dyn std::error::Error>;
	// type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;

	#[test]
	fn test_extract_blocks_single_rust_block() {
		let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
More text
        "#;

		let blocks = extract_blocks(content, Some("rust"));
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
	}

	#[test]
	fn test_extract_blocks_no_lang_specified() {
		let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
```python
def hello():
		print("Hello, world!")
```
        "#;

		let blocks = extract_blocks(content, None);
		assert_eq!(blocks.len(), 2);
		assert_eq!(blocks[0], "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
		assert_eq!(blocks[1], "def hello():\n\t\tprint(\"Hello, world!\")\n");
	}

	#[test]
	fn test_extract_blocks_single_python_block() {
		let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
```python
def hello():
		print("Hello, world!")
```
        "#;

		let blocks = extract_blocks(content, Some("python"));
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], "def hello():\n\t\tprint(\"Hello, world!\")\n");
	}

	#[test]
	fn test_extract_blocks_no_matching_blocks() {
		let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
        "#;

		let blocks = extract_blocks(content, Some("python"));
		assert_eq!(blocks.len(), 0);
	}

	#[test]
	fn test_extract_blocks_multiple_rust_blocks() {
		let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
More text
```rust
fn greet() {
		println!("Greetings!");
}
```
        "#;

		let blocks = extract_blocks(content, Some("rust"));
		assert_eq!(blocks.len(), 2);
		assert_eq!(blocks[0], "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
		assert_eq!(blocks[1], "fn greet() {\n\t\tprintln!(\"Greetings!\");\n}\n");
	}

	#[test]
	fn test_extract_blocks_handles_empty_blocks() {
		let content = r#"
Some text
```rust
```
```python
def hello():
		print("Hello, world!")
```
        "#;

		let blocks = extract_blocks(content, Some("rust"));
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0], "\n"); // Expecting an empty block
	}

	#[test]
	fn test_extract_blocks_no_code_blocks() {
		let content = r#"
This is a text without any code blocks.
        "#;

		let blocks = extract_blocks(content, None);
		assert_eq!(blocks.len(), 0);
	}
}

// endregion: --- Tests
