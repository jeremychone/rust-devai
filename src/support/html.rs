//! HTML Utilities
use crate::{Error, Result};
use html5ever::driver::ParseOpts;
use html5ever::parse_document;
use html5ever::serialize::SerializeOpts;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};

/// unescape code (sometime chatgpt encode the < and such)
pub fn decode_html_entities(content: &str) -> String {
	html_escape::decode_html_entities(&content).to_string()
}

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
/// * `html_content` - A `String` containing the HTML content to be processed.
///
/// # Returns
///
/// A `Result<String>` which is:
/// - `Ok(String)` containing the cleaned HTML content.
/// - `Err` if any parsing or serialization errors occur.
///
/// # Examples
///
/// ```
/// let html_content = r#"
/// <html>
/// <head>
/// <style>body { color: red; }</style>
/// </head>
/// <body>
/// <h1>Hello, World!</h1>
/// <script>alert('Hi');</script>
/// </body>
/// </html>
/// "#.to_string();
///
/// let cleaned_html = prune_to_content(html_content).unwrap();
/// assert!(cleaned_html.contains("<h1>Hello, World!</h1>"));
/// assert!(!cleaned_html.contains("<style>"));
/// assert!(!cleaned_html.contains("<script>"));
/// ```
pub fn prune_to_content(html_content: String) -> Result<String> {
	let dom = parse_document(RcDom::default(), ParseOpts::default())
		.from_utf8()
		.read_from(&mut html_content.as_bytes())?;

	remove_non_content_tags(&dom.document)?;

	let document: SerializableHandle = dom.document.clone().into();
	let serialize_opts = SerializeOpts::default();

	let mut output = Vec::new();
	html5ever::serialize(&mut output, &document, serialize_opts)?;

	let content = String::from_utf8(output).map_err(|err| Error::cc("html5ever serialization non utf8", err))?;
	let content = remove_empty_lines(content)?;

	Ok(content)
}

/// Removes empty lines from the given content, returning the cleaned string.
fn remove_empty_lines(content: String) -> Result<String> {
	let lines: Vec<&str> = content.lines().filter(|line| !line.trim().is_empty()).collect();
	Ok(lines.join("\n"))
}

/// Recursively removes non-content elements (like scripts and styles) from the DOM tree.
fn remove_non_content_tags(handle: &Handle) -> Result<()> {
	let tags_to_remove = &["script", "link", "style", "svg"];
	let mut indices_to_remove = Vec::new();

	for (index, child) in handle.children.borrow().iter().enumerate() {
		match &child.data {
			NodeData::Element { ref name, .. } => {
				let tag = name.local.as_ref();
				if tags_to_remove.contains(&tag) || !has_text_content(child) {
					indices_to_remove.push(index);
				} else {
					remove_tag_attributes(child)?;
					remove_non_content_tags(child)?;
				}
			}
			NodeData::Comment { .. } => {
				indices_to_remove.push(index);
			}
			_ => remove_non_content_tags(child)?,
		}
	}

	for &index in indices_to_remove.iter().rev() {
		handle
			.children
			.try_borrow_mut()
			.map_err(|err| Error::cc("html5ever child already borrowed", err))?
			.remove(index);
	}

	Ok(())
}

/// Retains only specific attributes (class, aria-label, href) in the given element.
fn remove_tag_attributes(handle: &Handle) -> Result<()> {
	if let NodeData::Element { ref attrs, .. } = handle.data {
		let mut attributes = attrs.borrow_mut();
		attributes.retain(|attr| {
			let name = attr.name.local.as_ref();
			name == "class" || name == "aria-label" || name == "href"
		});
	}
	Ok(())
}

/// Checks if the given node or its children contain any text content.
/// NOTE: This needs to be optimize, each node descend to the whole tree.
fn has_text_content(handle: &Handle) -> bool {
	match handle.data {
		NodeData::Text { ref contents } => !contents.borrow().trim().is_empty(),
		NodeData::Element { .. } => {
			for child in handle.children.borrow().iter() {
				if has_text_content(child) {
					return true;
				}
			}
			false
		}
		_ => false,
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_html_prune_to_content_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Simple HTML Page</title>
		<style> body{ color: red }</style>
		<link >
</head>
<body>
		<div>
			<span></span>
		</div>
    <h1 funky-attribute>Hello, World!</h1>
    <p>This is a simple HTML page.</p>
		<!-- Some Comment -->
</body>
</html>	
		"#;

		// -- Exec
		let html = prune_to_content(fx_html.to_string())?;

		// -- Check
		assert!(!html.contains("span"), "should not contain span");
		assert!(!html.contains("meta"), "should not contain meta");
		assert!(!html.contains("style"), "should not contain style");
		assert!(!html.contains("link"), "should not contain link");
		assert!(!html.contains("funky-attribute"), "should not contain funky-attribute");
		assert!(!html.contains("<!--"), "should not contain <!--");
		assert!(html.contains("Hello, World!"), "should contain Hello, World!");
		assert!(
			html.contains("<title>Simple HTML Page</title>"),
			"should contain the title"
		);

		Ok(())
	}
}

// endregion: --- Tests
