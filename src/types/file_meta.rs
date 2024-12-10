use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::{SFile, SPath};

#[derive(Debug, Serialize)]
pub struct FileMeta {
	name: String,
	path: String,
	stem: String,
	ext: String,
}

impl From<&SPath> for FileMeta {
	fn from(file: &SPath) -> Self {
		FileMeta {
			name: file.name().to_string(),
			path: file.to_string(),
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
		}
	}
}

impl From<SPath> for FileMeta {
	fn from(spath: SPath) -> Self {
		FileMeta {
			name: spath.name().to_string(),
			path: spath.to_string(),
			stem: spath.stem().to_string(),
			ext: spath.ext().to_string(),
		}
	}
}

impl From<SFile> for FileMeta {
	fn from(file: SFile) -> Self {
		FileMeta {
			name: file.name().to_string(),
			path: file.to_string(),
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
		}
	}
}

// region:    --- Lua

impl IntoLua for FileMeta {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("path", self.path)?;
		table.set("name", self.name)?;
		table.set("stem", self.stem)?;
		table.set("ext", self.ext)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
