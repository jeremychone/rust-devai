use crate::types::MdSection;
use mlua::IntoLua;

#[derive(Debug)]
pub struct MdSectionSplit {
	pub(super) before: String,
	pub(super) first: Option<MdSection>,
	pub(super) after: String,
}

impl IntoLua for MdSectionSplit {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("before", self.before)?;
		table.set("first", self.first)?;
		table.set("after", self.after)?;

		Ok(mlua::Value::Table(table))
	}
}
