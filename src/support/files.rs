use crate::{Error, Result};
use simple_fs::{SFile, SPath};
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

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

/// Returns the current dir
pub fn current_dir() -> Result<SPath> {
	let dir = env::current_dir().map_err(|err| Error::cc("Current dir error", err))?;
	let dir = SPath::new(dir)?;
	Ok(dir)
}

/// Relatively efficient way to determine if a file is empty, meaning lenght == 0, or only whitespace.
pub fn is_file_empty(file_path: impl AsRef<Path>) -> Result<bool> {
	let path = file_path.as_ref();
	let file = File::open(path).map_err(|err| {
		//
		Error::cc(
			"Cannot determine if file empty",
			format!("File '{}' open error. Cause: {err}", path.to_string_lossy()),
		)
	})?;
	let mut reader = BufReader::new(file);

	// First read with a small buffer of 64 bytes
	let mut small_buffer = [0; 64];
	let num_bytes = reader.read(&mut small_buffer)?;
	if num_bytes == 0 {
		return Ok(true);
	}
	if !is_buff_empty(&small_buffer[..num_bytes]) {
		return Ok(false);
	}
	// If we read less than the small buffer size, we've reached the end of the file.
	if num_bytes < small_buffer.len() {
		return Ok(true);
	}

	// Subsequent reads with a larger buffer of 1024 bytes
	let mut large_buffer = [0; 1024];
	loop {
		let num_bytes = reader.read(&mut large_buffer)?;
		if num_bytes == 0 {
			break;
		}
		if !is_buff_empty(&large_buffer[..num_bytes]) {
			return Ok(false);
		}
	}
	Ok(true)
}

fn is_buff_empty(buff: &[u8]) -> bool {
	let s = std::str::from_utf8(buff).unwrap_or("");
	s.chars().all(|c| c.is_whitespace())
}
