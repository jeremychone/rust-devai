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
	/// The path, might and will probably be relative
	pub path: String,
	/// The name of the file with extension e.g., `main.rs`
	pub name: String,
	/// Stem
	pub stem: String,
	/// Empty if there is no extension
	pub ext: String,
	/// The full text content of the file
	pub content: String,
}

/// Constructors
impl FileRecord {
	pub fn new(path: impl AsRef<Path>) -> Result<Self> {
		let sfile = SFile::from_path(path.as_ref())?;
		Self::from_sfile(sfile)
	}

	pub fn from_sfile(sfile: SFile) -> Result<Self> {
		let content = read_to_string(&sfile)?;
		Ok(FileRecord {
			path: sfile.to_string(),
			name: sfile.name().to_string(),
			stem: sfile.stem().to_string(),
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
		map.insert("stem".into(), file.stem.into());
		map.insert("ext".into(), file.ext.into());
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
