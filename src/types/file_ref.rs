use serde::Serialize;
use simple_fs::SFile;

#[derive(Serialize)]
pub struct FileRef {
	name: String,
	path: String,
	stem: String,
	ext: String,
}

impl From<SFile> for FileRef {
	fn from(file: SFile) -> Self {
		FileRef {
			name: file.file_name().to_string(),
			path: file.to_string(),
			stem: file.file_stem().to_string(),
			ext: file.ext().to_string(),
		}
	}
}
