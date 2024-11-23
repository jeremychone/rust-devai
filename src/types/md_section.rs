use crate::types::MdHeading;
use mlua::IntoLua;

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

	#[allow(unused)]
	pub fn heading(&self) -> Option<&MdHeading> {
		self.heading.as_ref()
	}
}

// region:    --- Lua

impl IntoLua for MdSection {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("content", self.content)?;
		table.set("heading", self.heading)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
