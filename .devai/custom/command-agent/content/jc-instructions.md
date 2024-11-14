
# Markers Instruction

> This will be notes that will be excluded in the `file::read_md_section(...) -> String`

- In the code above, instruction sections are delimited by `<<START>>` and `<<END>>`. These will be called instruction sections.

- They may span multiple lines for a given instruction section.

- For each instruction section:
    - Provide the new content for that instruction section.
    - Return only the result of the instruction sections, in the same order.
    - Put the result in a markdown block with the language `ai-answer` in their respective order.

- If here is no marker, that's fine, do not answer anything, just "No markers here, nothing to do."

- Ensure the order of the instruction sections is respected.

# Rhai Doc Instruction

When asked to update or add documentation to a Rhai Doc. The two following parts need to be updated. 

First, in the the module comment (i.e. `//!`) section, make sure you update the `### Functions` part with all of the rhai functions like: 

```rust
//! ### Functions
//! * `html::prune_to_content(html_content: string) -> string`
```

Remove the one that are not in the file. 

Then, on top of each of the rust functions that is mapped to one or more rhai module function, update to follow the following format: 

```rust
/// ## RHAI Documentation
/// ```rhai
/// html::prune_to_content(html_content: string) -> string
/// ```
///
/// Strips non-content elements from the provided HTML content and returns the cleaned HTML as a string.
///
/// This function removes:
/// - Non-visible tags such as `<script>`, `<link>`, `<style>`, and `<svg>`.
/// - HTML comments.
/// - Empty lines.
/// - Attributes except for `class`, `aria-label`, and `href`.
///
/// # Arguments
///
/// * `html_content` - (required) A `String` containing the HTML content to be processed.
///
/// # Returns
///
/// A `String` containing the cleaned HTML content.
///
fn prune_to_content_rhai(html_content: &str) -> RhaiResult {
	match prune_to_content(html_content.to_string()) {
		Ok(cleaned_html) => Ok(Dynamic::from(cleaned_html)),
		Err(err) => Err(Box::new(EvalAltResult::ErrorRuntime(
			format!("Failed to prune HTML content: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}
```

