use crate::script::{DynamicMap, IntoDynamic};
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
	#[allow(unused)]
	pub fn new(lang: Option<String>, content: impl Into<String>) -> Self {
		MdBlock {
			lang,
			content: content.into(),
		}
	}
}

// region:    --- Dynamic Froms

impl IntoDynamic for MdBlock {
	/// Converts the `MdBlock` instance into a Rhai `Dynamic` type.
	/// Note: Kind of needed, because Dynamic has a `from(..)` which make the into a little inconvenient.
	fn into_dynamic(self) -> Dynamic {
		let map = DynamicMap::default().insert("lang", self.lang).insert("content", self.content);
		map.into_dynamic()
	}
}

// endregion: --- Dynamic Froms
