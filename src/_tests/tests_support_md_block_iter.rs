use crate::support::Extrude;
use crate::support::md::MdBlockIter;
use crate::types::MdBlock;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

#[test]
fn test_md_block_iter_single_rust_block_simple() -> Result<()> {
	let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
More text
        "#;

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, Some("rust"), None).collect();
	assert_eq!(blocks.len(), 1);
	assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
	assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");

	Ok(())
}

#[test]
fn test_md_block_iter_single_rust_block_extrude_content() -> Result<()> {
	let content = r#"
Some text

```rust
fn main() {
		println!("Hello, world!");
}
```
More text
"#;

	let (blocks, content) =
		MdBlockIter::new(content, Some("rust"), Some(Extrude::Content)).collect_blocks_and_extruded_content();
	assert_eq!(
		content,
		"
Some text

More text
",
	);
	assert_eq!(blocks.len(), 1);
	assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
	assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");

	Ok(())
}

#[test]
fn test_md_block_iter_no_lang_specified() -> Result<()> {
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

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, None, None).collect();
	assert_eq!(blocks.len(), 2);
	assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
	assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
	assert_eq!(blocks[1].lang.as_ref().expect("should have lang"), "python");
	assert_eq!(blocks[1].content, "def hello():\n\t\tprint(\"Hello, world!\")\n");

	Ok(())
}

#[test]
fn test_md_block_iter_single_python_block() -> Result<()> {
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

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, Some("python"), None).collect();
	assert_eq!(blocks.len(), 1);
	assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "python");
	assert_eq!(blocks[0].content, "def hello():\n\t\tprint(\"Hello, world!\")\n");

	Ok(())
}

#[test]
fn test_md_block_iter_no_matching_blocks() -> Result<()> {
	let content = r#"
Some text
```rust
fn main() {
		println!("Hello, world!");
}
```
        "#;

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, Some("python"), None).collect();
	assert_eq!(blocks.len(), 0);

	Ok(())
}

#[test]
fn test_md_block_iter_match_empty_lang() -> Result<()> {
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

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, Some(""), None).collect();
	assert_eq!(blocks.len(), 1);
	assert_eq!(blocks[0].lang.as_deref(), Some(""));
	assert_eq!(blocks[0].content, "Some content of empty lang block\n");

	Ok(())
}

#[test]
fn test_md_block_iter_multiple_rust_blocks() -> Result<()> {
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

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, Some("rust"), None).collect();
	assert_eq!(blocks.len(), 2);
	assert_eq!(blocks[0].lang.as_ref().expect("should have lang"), "rust");
	assert_eq!(blocks[0].content, "fn main() {\n\t\tprintln!(\"Hello, world!\");\n}\n");
	assert_eq!(blocks[1].lang.as_ref().expect("should have lang"), "rust");
	assert_eq!(blocks[1].content, "fn greet() {\n\t\tprintln!(\"Greetings!\");\n}\n");

	Ok(())
}

#[test]
fn test_md_block_iter_handles_empty_blocks() -> Result<()> {
	let content = r#"
Some text
```rust
```
```python
def hello():
		print("Hello, world!")
```
        "#;

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, Some("rust"), None).collect();
	assert_eq!(blocks.len(), 1);
	assert_eq!(blocks[0].content, ""); // Expecting an empty block

	Ok(())
}

#[test]
fn test_md_block_iter_no_code_blocks() -> Result<()> {
	let content = r#"
This is a text without any code blocks.
        "#;

	let blocks: Vec<MdBlock> = MdBlockIter::new(content, None, None).collect();
	assert_eq!(blocks.len(), 0);

	Ok(())
}
