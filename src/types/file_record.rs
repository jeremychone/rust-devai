use crate::script::DynamicMap;
use crate::Result;
use rhai::Dynamic;
use serde::Serialize;
use simple_fs::SFile;
use std::fs::read_to_string;
use std::path::Path;

/// FileRecord contains the metadata information about the file (name, ext, etc.) as well as the content.
#[derive(Serialize)]
pub struct FileRecord {
	/// The name of the file with extension e.g., `main.rs`
	pub name: String,
	/// The path, might and will probably be relative
	pub path: String,
	/// Empty if there is no extension
	pub ext: String,
	/// Stem
	pub stem: String,
	/// The full text content of the file
	pub content: String,
}

/// Constructors
impl FileRecord {
	pub fn new(path: impl AsRef<Path>) -> Result<Self> {
		let sfile = SFile::from_path(path.as_ref())?;
		let content = read_to_string(&sfile)?;
		Ok(FileRecord {
			name: sfile.file_name().to_string(),
			path: sfile.to_string(),
			stem: sfile.file_stem().to_string(),
			ext: sfile.ext().to_string(),
			content,
		})
	}
}

// region:    --- Rhai Dynamic From/To

// Implement conversion from File to Dynamic
impl From<FileRecord> for Dynamic {
	fn from(file: FileRecord) -> Dynamic {
		let mut map = rhai::Map::new();
		map.insert("name".into(), file.name.into());
		map.insert("path".into(), file.path.into());
		map.insert("content".into(), file.content.into());
		Dynamic::from_map(map)
	}
}

// Implement conversion from Dynamic to File
impl TryFrom<Dynamic> for FileRecord {
	type Error = crate::Error;

	fn try_from(value: Dynamic) -> Result<Self> {
		let map = DynamicMap::new(value)?;
		Ok(FileRecord {
			name: map.get("name")?,
			path: map.get("path")?,
			stem: map.get("stem")?,
			ext: map.get("ext")?,
			content: map.get("content")?,
		})
	}
}

// endregion: --- Rhai Dynamic From/To
