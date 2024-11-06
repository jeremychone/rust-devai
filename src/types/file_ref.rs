use rhai::Dynamic;
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

// region:    --- Dynamic Froms

impl FileRef {
	/// Use this instead of the `From` trait, because Rhai's `Dynamic`
	/// `From` implementation requires cloning.
	/// Implementing `From` for `Dynamic` was confusing.
	pub fn into_dynamic(self) -> Dynamic {
		let mut map = rhai::Map::new();
		map.insert("path".into(), self.path.into());
		map.insert("name".into(), self.name.into());
		map.insert("stem".into(), self.stem.into());
		map.insert("ext".into(), self.ext.into());
		Dynamic::from_map(map)
	}
}

// endregion: --- Dynamic Froms
