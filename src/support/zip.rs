use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use camino::Utf8Path;
use camino::Utf8PathBuf;
use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::ZipArchive;
use zip::ZipWriter;
use zip::write::{FileOptions, SimpleFileOptions};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Creates a zip archive from the directory `src_dir` and writes it to `dest_file`.
///
/// `src_dir` is the directory to be zipped.
/// `dest_file` is the destination file path for the zip archive.
///
/// This function recursively adds files and subdirectories from `src_dir` to the zip archive.
pub fn zip_dir(src_dir: impl AsRef<Utf8Path>, dest_file: impl AsRef<Utf8Path>) -> Result<()> {
	let src_dir = src_dir.as_ref();
	let dest_file = dest_file.as_ref();

	// Create the destination zip file.
	let file = File::create(dest_file.as_std_path())?;
	let mut zip = ZipWriter::new(file);

	// Set default options with deflate compression.
	let options = SimpleFileOptions::default();

	// Walk through the directory.
	for entry in WalkDir::new(src_dir.as_std_path()) {
		let entry = entry?;
		let Some(path) = Utf8Path::from_path(entry.path()) else {
			continue;
		};

		let relative_path = path.strip_prefix(src_dir.as_std_path())?;

		// Convert relative path to UTF-8 string and normalize slashes for cross-platform compatibility.
		let name = relative_path.as_str().replace("\\", "/");

		if path.is_dir() {
			// Add directory entry to zip archive.
			if !name.is_empty() {
				// Ensure directory name ends with '/'.
				let dir_name = if name.ends_with('/') {
					name.to_string()
				} else {
					format!("{}/", name)
				};
				zip.add_directory(dir_name, options)?;
			}
		} else {
			// Add file entry to zip archive.
			zip.start_file(name, options)?;
			let mut f = File::open(path)?;
			io::copy(&mut f, &mut zip)?;
		}
	}

	zip.finish()?;
	Ok(())
}

/// Extracts the zip archive from `src_zip` into the directory `dest_dir`.
///
/// `src_zip` is the path to the zip archive.
/// `dest_dir` is the destination directory where the contents of the zip will be extracted.
pub fn unzip_file(src_zip: impl AsRef<Utf8Path>, dest_dir: impl AsRef<Utf8Path>) -> Result<()> {
	let src_zip = src_zip.as_ref();
	let dest_dir = dest_dir.as_ref();

	// Open the zip archive.
	let file = File::open(src_zip.as_std_path())?;
	let mut archive = ZipArchive::new(file)?;

	// Iterate over zip entries.
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		let outpath = dest_dir.join(file.name());

		if file.name().ends_with('/') {
			// Create the directory if it doesn't exist.
			fs::create_dir_all(outpath.as_std_path())?;
		} else {
			// Ensure parent directory exists.
			if let Some(parent) = outpath.parent() {
				fs::create_dir_all(parent.as_std_path())?;
			}
			// Create and write the file.
			let mut outfile = File::create(outpath.as_std_path())?;
			io::copy(&mut file, &mut outfile)?;
		}
	}

	Ok(())
}
