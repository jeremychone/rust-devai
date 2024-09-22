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
		assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
		assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
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
		assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
		assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
		assert_eq!(blocks[1].lang.as_ref().expect("should have lang"), "python");
		assert_eq!(blocks[1].content, "def hello():\n\t\tprint(\"Hello, world!\")\n");
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
		assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "python");
		assert_eq!(blocks[0].content, "def hello():\n\t\tprint(\"Hello, world!\")\n");
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
	fn test_extract_blocks_match_empty_lang() {
		let content = r#"
Some text

```
Some content of empty lang block
```

```rust
fn main() {
		println!("Hello, world!");
}
```




        "#;

		let blocks = extract_blocks(content, Some(""));
		assert_eq!(blocks.len(), 1);
		assert_eq!(blocks[0].lang.as_deref(), Some(""));
		assert_eq!(blocks[0].content, "Some content of empty lang block\n");
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
		assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
		assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
		assert_eq!(blocks[1].lang.as_ref().expect("should have lang"), "rust");
		assert_eq!(blocks[1].content, "fn greet() {\n\t\tprintln!(\"Greetings!\");\n}\n");
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
		assert_eq!(blocks[0].content, "\n"); // Expecting an empty block
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
