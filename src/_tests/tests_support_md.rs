type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;

#[test]
fn test_extract_blocks_single_rust_block() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_no_lang_specified() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_single_python_block() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_no_matching_blocks() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_match_empty_lang() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_multiple_rust_blocks() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_handles_empty_blocks() -> Result<()> {
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

	Ok(())
}

#[test]
fn test_extract_blocks_no_code_blocks() -> Result<()> {
	let content = r#"
This is a text without any code blocks.
        "#;

	let blocks = extract_blocks(content, None);
	assert_eq!(blocks.len(), 0);

	Ok(())
}
