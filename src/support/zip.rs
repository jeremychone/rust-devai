use crate::{Error, Result};
use camino::Utf8Path;
use simple_fs::SPath;
use std::fs::{self, File};
use std::io::{self, Read as _};
use walkdir::WalkDir;
use zip::ZipArchive;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

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
		let entry = entry.map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail to zip directory. Error on entry. Cause {err}"),
		})?;
		let Some(path) = Utf8Path::from_path(entry.path()) else {
			continue;
		};

		let relative_path = path.strip_prefix(src_dir).map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail strip_prefix '{src_dir}' on '{path}'. Cause {err}"),
		})?;

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
				zip.add_directory(&dir_name, options).map_err(|err| Error::ZipFail {
					zip_dir: src_dir.to_string(),
					cause: format!("Fail add directory '{dir_name}'. Cause {err}"),
				})?;
			}
		} else {
			// Add file entry to zip archive.
			zip.start_file(&name, options).map_err(|err| Error::ZipFail {
				zip_dir: src_dir.to_string(),
				cause: format!("Fail zip.start_file '{name}'. Cause {err}"),
			})?;
			let mut f = File::open(path)?;
			io::copy(&mut f, &mut zip)?;
		}
	}

	zip.finish().map_err(|err| Error::ZipFail {
		zip_dir: src_dir.to_string(),
		cause: format!("Fail zip.finish '{src_dir}'. Cause {err}"),
	})?;
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
	let mut archive = ZipArchive::new(file).map_err(|err| Error::UnzipZipFail {
		zip_file: src_zip.to_string(),
		cause: format!("Fail to create new archive. Cause: {err}"),
	})?;

	// Iterate over zip entries.
	for i in 0..archive.len() {
		let mut file = archive.by_index(i).map_err(|err| Error::UnzipZipFail {
			zip_file: src_zip.to_string(),
			cause: format!("Fail to get item by_index {i}. Cause: {err}"),
		})?;
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

pub fn extract_text_content(src_zip_path: impl AsRef<SPath>, content_path: &str) -> Result<String> {
	let src_zip_path = src_zip_path.as_ref();
	let file = File::open(src_zip_path)?;

	let mut archive = ZipArchive::new(file).map_err(|err| Error::Zip {
		zip_file: src_zip_path.name().to_string(),
		cause: err.to_string(),
	})?;

	let mut file = archive.by_name(content_path).map_err(|_| Error::ZipFileNotFound {
		zip_file: src_zip_path.name().to_string(),
		content_path: content_path.to_string(),
	})?;

	let mut data: Vec<u8> = Vec::new();
	file.read_to_end(&mut data)?;
	let content = String::from_utf8(data).map_err(|err| Error::ZipContent {
		zip_file: src_zip_path.name().to_string(),
		content_path: content_path.to_string(),
		cause: format!("Not utf8. Info: {err}"),
	})?;

	Ok(content)
}
