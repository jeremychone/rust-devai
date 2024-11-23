use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::{SFile, SPath};

#[derive(Debug, Serialize)]
pub struct FileRef {
	name: String,
	path: String,
	stem: String,
	ext: String,
}

impl From<&SPath> for FileRef {
	fn from(file: &SPath) -> Self {
		FileRef {
			name: file.name().to_string(),
			path: file.to_string(),
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
		}
	}
}

impl From<SPath> for FileRef {
	fn from(spath: SPath) -> Self {
		FileRef {
			name: spath.name().to_string(),
			path: spath.to_string(),
			stem: spath.stem().to_string(),
			ext: spath.ext().to_string(),
		}
	}
}

impl From<SFile> for FileRef {
	fn from(file: SFile) -> Self {
		FileRef {
			name: file.name().to_string(),
			path: file.to_string(),
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
		}
	}
}

// region:    --- Lua

impl IntoLua for FileRef {
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
