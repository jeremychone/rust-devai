use crate::Result;

pub fn clean_rust_content(content: String) -> Result<String> {
	let trimmed_content = content.trim();

	// Check if the content starts with ```rust and ends with ```
	if trimmed_content.starts_with("```rust") && trimmed_content.ends_with("```") {
		// Remove the first ```rust and the last ```
		let without_rust_tag = &trimmed_content["```rust".len()..];
		let cleaned_content = &without_rust_tag[..without_rust_tag.len() - 3]; // Remove the ending ```

		// Return the cleaned content, trimmed of leading/trailing whitespace
		Ok(cleaned_content.trim().to_string())
	} else {
		Ok(trimmed_content.to_string())
	}
}
