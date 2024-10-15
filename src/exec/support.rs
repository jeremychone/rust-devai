use crate::Result;
use simple_fs::{SFile, SPath};
use std::path::{Path, PathBuf};

/// Returns the file that matches the path for a given list of directories.
/// This is useful for finding a file path with some directory precedence rules.
pub fn first_file_from_dirs(dirs: &[&str], path: &str) -> Result<Option<SFile>> {
	for dir in dirs {
		let file_path = Path::new(dir).join(path);
		if file_path.exists() {
			return Ok(Some(SFile::from_path(file_path)?));
		}
	}

	Ok(None)
}

/// Returns the (solo_path, target_path) tuple for a file path of either.
/// - If the path ends with `.devai`, then it is the solo path.
/// - Otherwise, add `.devai` to the file name in the same path.
pub fn get_solo_and_target_path(path: impl Into<PathBuf>) -> Result<(SPath, SPath)> {
	let path = SPath::new(path)?;

	// returns (solo_path, target_path)
	// path is the solo_path
	let solo_and_target_path = if path.ext() == "devai" {
		let target_path = path.new_sibling(path.file_stem())?;
		(path, target_path)
	}
	// path is the target_path
	else {
		let solo_path = path.new_sibling(format!("{}.devai", path.file_name()))?;
		(solo_path, path)
	};

	Ok(solo_and_target_path)
}
