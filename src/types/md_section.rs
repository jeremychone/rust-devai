use crate::script::{DynaMap, IntoDynamic};
use crate::types::MdHeading;
use rhai::Dynamic;

#[derive(Debug)]
pub struct MdSection {
	pub content: String,
	pub heading: Option<MdHeading>,
}

/// Constructors
/// For now, constructe by
#[allow(unused)]
impl MdSection {
	pub fn from_content(content: impl Into<String>) -> Self {
		Self {
			content: content.into(),
			heading: None,
		}
	}
	pub fn new(content: String, heading: impl Into<Option<MdHeading>>) -> Self {
		Self {
			content,
			heading: heading.into(),
		}
	}
}

/// Getters
impl MdSection {
	#[allow(unused)]
	pub fn content(&self) -> &str {
		&self.content
	}

	pub fn heading(&self) -> Option<&MdHeading> {
		self.heading.as_ref()
	}
}

/// Transformers
// impl MdSection {
// 	pub fn into_content(self) -> String {
// 		self.content
// 	}
// 	pub fn into_content_and_heading(self) -> (String, Option<MdHeading>) {
// 		(self.content, self.heading)
// 	}
// }

// region:    --- IntoDynamic

impl IntoDynamic for MdSection {
	fn into_dynamic(self) -> Dynamic {
		let map = DynaMap::default()
			.insert("heading_level", self.heading().map(|h| h.level()))
			.insert("heading_name", self.heading().map(|h| h.name()))
			.insert("heading_content", self.heading.map(|h| h.into_content()))
			.insert("content", self.content);

		map.into_dynamic()
	}
}
// endregion: --- IntoDynamic
