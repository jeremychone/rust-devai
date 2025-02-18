use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::{SFile, SPath};

#[derive(Debug, Serialize)]
pub struct FileMeta {
	path: String,
	/// The dir/parent path of this file from path (will be empty if no parent of the rel path)
	dir: String,
	name: String,
	stem: String,
	ext: String,
}

impl From<&SPath> for FileMeta {
	fn from(file: &SPath) -> Self {
		let dir = file.parent().map(|p| p.to_string()).unwrap_or_default();
		FileMeta {
			path: file.to_string(),
			name: file.name().to_string(),
			dir,
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
		}
	}
}

impl From<SPath> for FileMeta {
	fn from(spath: SPath) -> Self {
		let dir = spath.parent().map(|p| p.to_string()).unwrap_or_default();
		FileMeta {
			path: spath.to_string(),
			name: spath.name().to_string(),
			dir,
			stem: spath.stem().to_string(),
			ext: spath.ext().to_string(),
		}
	}
}

impl From<SFile> for FileMeta {
	fn from(file: SFile) -> Self {
		let dir = file.parent().map(|p| p.to_string()).unwrap_or_default();
		FileMeta {
			path: file.to_string(),
			dir,
			name: file.name().to_string(),
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
		table.set("dir", self.dir)?;
		table.set("name", self.name)?;
		table.set("stem", self.stem)?;
		table.set("ext", self.ext)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
