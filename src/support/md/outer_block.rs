use std::borrow::Cow;

/// Without parsing the markdown, If the string starts with a  ` ``` ` then it will remove that line and
/// the keep the content only until the last ` ```
///
/// # Arguments
///
/// * `content` - A string slice that holds the input text.
///
/// # Returns
///
/// A `String` containing the extracted content between the first and last ```
pub fn outer_block_content_or_raw(content: &str) -> Cow<str> {
	if !content.starts_with("```") {
		return content.into();
	}

	// Split the input content into lines for line-by-line processing.
	let lines: Vec<&str> = content.lines().collect();

	// Find the index of the first line that starts with ``` (ignoring leading whitespace).
	let first_backtick = lines.iter().position(|line| line.starts_with("```"));

	// Find the index of the last line that starts with ``` (ignoring leading whitespace).
	let last_backtick = lines.iter().rposition(|line| line.starts_with("```"));

	// Check if both opening and closing backticks are found and are distinct.
	if let (Some(start), Some(end)) = (first_backtick, last_backtick) {
		// Ensure that the first backtick is before the last backtick.
		if start < end {
			// Extract the lines between the first and last backtick lines.
			let extracted_lines = &lines[start + 1..end];
			// Join the extracted lines back into a single string separated by newlines.
			let mut content = extracted_lines.join("\n");
			content.push('\n');
			return content.into();
		}
	}

	// If backticks are not properly found, return the original content.
	content.into()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_md_outer_block_content_simple() {
		// -- Fixtures
		let input = "\
Here is some text before the code block.

```
fn main() {
    println!(\"Hello, world!\");
}
```

Here is some text after the code block.";
		let expected = "fn main() {\n    println!(\"Hello, world!\");\n}\n";

		// -- Exec
		let result = outer_block_content_or_raw(input);

		// -- Check
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_with_language() {
		// -- Fixtures
		let input = "\
Start of the text.

```python
def hello():
    print(\"Hello, Python!\")
```

End of the text.";

		// -- Exec
		let result = outer_block_content_or_raw(input);

		// -- Check
		let expected = "def hello():\n    print(\"Hello, Python!\")\n";
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_multiple_code_blocks() {
		// -- Fixtures
		let fx_content = "
console.log(\"First code block\");
```

Some intermediate text.

Second code block:

```rust
fn main() {
    println!(\"Second code block\");
}";
		let input = format!(
			"
First code block:

```javascript
{fx_content}
```

End of the text."
		);

		// -- Exec
		let result = outer_block_content_or_raw(&input);

		// -- Check
		assert_eq!(result, format!("{fx_content}\n"));
	}

	#[test]
	fn test_md_outer_block_content_no_backticks() {
		// -- Fixtures
		let input = "This is regular text without any code blocks.";
		let expected = "This is regular text without any code blocks.";

		// -- Exec
		let result = outer_block_content_or_raw(input);

		// -- Check
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_only_opening_backticks() {
		// -- Fixtures
		let input = "\
Text before the code block.

```
fn incomplete() {
    // Missing closing backticks
}";
		let expected = "\
Text before the code block.

```
fn incomplete() {
    // Missing closing backticks
}";

		// -- Exec
		let result = outer_block_content_or_raw(input);

		// -- Check
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_only_closing_backticks() {
		// -- Fixtures
		let input = "\
Missing opening backticks for this code block.

fn incomplete() {
    // Missing opening backticks
}
```
";
		let expected = "\
Missing opening backticks for this code block.

fn incomplete() {
    // Missing opening backticks
}
```
";

		// -- Exec
		let result = outer_block_content_or_raw(input);

		// -- Check
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_adjacent_backticks() {
		// -- Fixtures
		let input = "\
Text before.

```
```

Text after.";
		let expected = "\n";

		// -- Exec
		let result = outer_block_content_or_raw(input);

		// -- Check
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_with_whitespace() {
		// -- Fixtures
		let fx_input = "
Text before.

    ```
    Line within code block with leading whitespace.
    ```

Text after.";
		let expected = fx_input.to_string();

		// -- Exec
		let result = outer_block_content_or_raw(fx_input);

		// -- Check
		assert_eq!(result, expected);
	}

	#[test]
	fn test_md_outer_block_content_with_inner_backticks() {
		// -- Fixtures
		let fx_content = "Here is some code with backticks:
let s = \"Hello, `world`!\";";
		let input = format!(
			"\
Start text.

```
{}
```

End text.",
			fx_content
		);

		// -- Exec
		let result = outer_block_content_or_raw(&input);

		// -- Check
		assert_eq!(result, format!("{fx_content}\n"));
	}
}
