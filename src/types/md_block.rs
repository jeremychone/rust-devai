use rhai::Dynamic;
use serde::Serialize;

/// Represents a Markdown block with optional language and content.
#[derive(Debug, Serialize)]
pub struct MdBlock {
	pub lang: Option<String>,
	pub content: String,
}

impl MdBlock {
	/// Creates a new `MdBlock` with the specified language and content.
	pub fn new(lang: Option<String>, content: impl Into<String>) -> Self {
		MdBlock {
			lang,
			content: content.into(),
		}
	}
}

// region:    --- Dynamic Froms

impl MdBlock {
	/// Converts the `MdBlock` instance into a Rhai `Dynamic` type.
	/// Note: Kind of needed, becuase Dynamic has a `from(..)` which make the into a little inconvenient.
	pub fn into_dynamic(self) -> Dynamic {
		let mut map = rhai::Map::new();
		map.insert("lang".into(), self.lang.map_or_else(|| Dynamic::UNIT, Dynamic::from));
		map.insert("content".into(), self.content.into());
		Dynamic::from_map(map)
	}
}

// endregion: --- Dynamic Froms
