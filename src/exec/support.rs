use crate::hub::get_hub;
use crate::{Error, Result};
use simple_fs::SFile;
use std::path::Path;
use std::process::Command;

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

pub async fn open_vscode(path: &Path) {
	let output = Command::new("code")
		.arg(path)
		.output()
		.expect("Failed to execute VSCode 'code' command");

	if !output.status.success() {
		let msg = format!("Error opening VSCode: {}", String::from_utf8_lossy(&output.stderr));
		get_hub().publish(Error::Custom(msg.to_string())).await;
	}
}
