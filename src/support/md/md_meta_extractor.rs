#![allow(unused)]
use crate::types::MdBlock;
use crate::Result;
use serde_json::Value;

pub fn extract_meta(content: &str) -> Result<(Value, String)> {
	let (meta_blocks, content) = extract_md_blocks_and_content(content, true)?;
	let content = content.unwrap_or_default();

	let value = merge_values(meta_blocks)?;

	Ok((value, content))
}

// region:    --- Block Value Parser

fn merge_values(meta_blocks: Vec<MdBlock>) -> Result<Value> {
	let mut values: Vec<Value> = Vec::new();

	// -- Capture the json Values
	for meta_block in meta_blocks {
		match meta_block.lang.as_deref() {
			Some("toml") => {
				let content = meta_block.content;
				let toml_value: toml::Value = content.parse()?;
				let json_value: Value = serde_json::to_value(toml_value)?;
				values.push(json_value);
			}
			Some(other) => return Err(format!("Lang '{other}' not supported for meta block").into()),
			None => return Err("Meta block must have a compatible lang".into()),
		}
	}

	// -- Merge the values into one
	// NOTE: Will assum ethat the values a object
	//       Does NOT do deep merge for now.
	let mut merged = serde_json::Map::new();
	for value in values {
		if let Value::Object(obj) = value {
			for (k, v) in obj {
				merged.insert(k, v);
			}
		}
	}
	let merged_value = Value::Object(merged);

	Ok(merged_value)
}

// endregion: --- Block Value Parser

// region:    --- Lexer

#[derive(Debug)]
enum Action {
	Initial,
	StartBlock,
	CloseBlock,
	CaptureInContent,
	CaptureInMetaBlock,
}

///
/// - `content` - The content to extract the meta blocks
/// - `extrude` - If true, will return the String (second tuple value) of the content minus the meta blocks.
///
/// Returns the merge root value (if at least one), and the content, without the `#!meta` code blocks.
fn extract_md_blocks_and_content(content: &str, extrude: bool) -> Result<(Vec<MdBlock>, Option<String>)> {
	let lines = content.lines();

	let mut content: Vec<&str> = Vec::new();
	let mut md_blocks: Vec<MdBlock> = Vec::new();

	// (lang, block_content_lines)
	type MetaBlock<'a> = (Option<String>, Vec<&'a str>);

	let mut current_meta_block: Option<MetaBlock> = Default::default();
	let mut in_block = false;
	let mut in_candidate_meta_block = false;
	let mut first_block_line = false;
	let mut action = Action::Initial;
	let mut previous_line: Option<&str> = None;

	for line in lines {
		// -- Determine Action
		// Getting in or out of a code block
		if line.starts_with("```") {
			first_block_line = false;
			if in_block {
				in_block = false;
				in_candidate_meta_block = false;
				action = Action::CloseBlock;
			} else {
				in_block = true;
				first_block_line = true;
				let is_meta_lang = line.strip_prefix("```").map(|v| v.trim() == "toml").unwrap_or_default();
				in_candidate_meta_block = is_meta_lang;
				action = Action::StartBlock;
			}
		}
		// Lines that are not ```
		else {
			// -- If in block
			if in_block {
				if in_candidate_meta_block {
					if first_block_line {
						if line.trim() == "#!meta" {
							first_block_line = false;
							action = Action::CaptureInMetaBlock
						} else {
							action = Action::CaptureInContent;
						}
					}
					//
					else {
						// Same action
					}
				} else {
					action = Action::CaptureInContent;
				}
			}
			// -- Not in block
			else {
				action = Action::CaptureInContent;
			}
		}

		// -- Process the Action
		match action {
			Action::Initial => {
				// Should never be here per logic
				// println!("INITIAL {action:?}");
			}
			Action::StartBlock => {
				// We do not know yet, needs to wait for next action.
			}
			Action::CloseBlock => {
				//
				match current_meta_block {
					Some(meta_block) => {
						let md_block = MdBlock {
							lang: meta_block.0,
							content: meta_block.1.join("\n"),
						};
						md_blocks.push(md_block);
						current_meta_block = None
					}
					None => content.push(line),
				}
			}
			Action::CaptureInContent => {
				if first_block_line {
					if let Some(prev_line) = previous_line {
						if extrude {
							content.push(prev_line);
						}
						// TODO: Should assess if we need to change state here, or implement a new Action::CaptureInContentAndPrevLine
						first_block_line = false;
					}
				}

				if extrude {
					content.push(line)
				}
			}
			Action::CaptureInMetaBlock => {
				//
				match current_meta_block.as_mut() {
					Some(meta_block) => meta_block.1.push(line),
					None => {
						//
						let lang = previous_line
							.and_then(|prev_line| prev_line.strip_prefix("```").map(|v| v.trim().to_string()));
						// type MetaBlock<'a> = (Option<String>, Vec<&'a str>);
						current_meta_block = Some((lang, Vec::new()))
					}
				}
			}
		}

		previous_line = Some(line);
	}

	let content = if extrude { Some(content.join("\n")) } else { None };

	Ok((md_blocks, content))
}

// endregion: --- Lexer

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use value_ext::JsonValueExt;

	// region:    --- fx_md
	const FX_MD_SIMPLE: &str = r#"
Hey some content over here. 

```toml
#!meta
model = "deepseek-chat"
files = ["src/**/*rs"]
```

This is is pretty cool

```toml
#!meta
temperature = 0.0
```

```toml
#!meta
```

And some more2

```toml
some = "stuff"
```

```python
#!meta
def some() 
 return 123
```


		"#;
	// endregion: --- fx_md

	#[test]
	fn test_meta_extrude_simple() -> Result<()> {
		// -- Exec
		let (value_root, content) = extract_meta(FX_MD_SIMPLE)?;

		// -- Check
		assert_eq!(value_root.x_get_f64("temperature")?, 0.0);
		assert_eq!(value_root.x_get_str("model")?, "deepseek-chat");
		let array = value_root.get("files").and_then(|v| v.as_array()).ok_or("Should have files")?;
		let strs = array.iter().map(|v| v.as_str().unwrap_or_default()).collect::<Vec<_>>();
		assert_eq!(&strs, &["src/**/*rs"]);

		// Content
		assert!(
			content.contains("Hey some content over here."),
			"Hey some content over here."
		);
		assert!(content.contains(r#"```toml"#), "```toml");
		assert!(content.contains(r#"some = "stuff""#), "some = stuff");

		Ok(())
	}

	#[test]
	fn test_extract_md_blocks_and_content_simple() -> Result<()> {
		// -- Exec
		let (meta_blocks, content) = extract_md_blocks_and_content(FX_MD_SIMPLE, true)?;

		// -- Check
		// assert meta blocks
		assert_eq!(meta_blocks.len(), 3);
		let meta_block = meta_blocks.first().ok_or("Should have at least one meta block")?;
		let lang = meta_block.lang.as_deref().ok_or("Meta block should have lang")?;
		assert_eq!(lang, "toml");
		assert!(
			meta_block.content.contains(r#"files = ["src/**/*rs"]"#),
			"should have files"
		);
		let meta_block = meta_blocks.get(1).ok_or("Should have at least thow meta block")?;
		assert!(
			meta_block.content.contains(r#"temperature = 0.0"#),
			"should have temperature"
		);
		// assert content
		let content = content.ok_or("Should have content")?;
		assert!(
			content.contains("Hey some content over here."),
			"Hey some content over here."
		);
		assert!(content.contains(r#"```toml"#), "```toml");
		assert!(content.contains(r#"some = "stuff""#), "some = stuff");
		assert!(content.contains(r#"```python"#), "```python");
		assert!(content.contains(r#"def some()"#), "def some()");
		assert!(content.contains("And some more2"), "And some more2");

		Ok(())
	}
}

// endregion: --- Tests
