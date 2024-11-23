use mlua::IntoLua;
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

// region:    --- Lua

impl IntoLua for MdBlock {
	/// Converts the `MdBlock` instance into a Lua Value
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("lang", self.lang)?;
		table.set("content", self.content)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
