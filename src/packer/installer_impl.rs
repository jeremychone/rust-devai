use crate::dir_context::DirContext;
use crate::packer::pack_toml::{PackToml, parse_validate_pack_toml};
use crate::support::zip;
use crate::{Error, Result};
use reqwest::Client;
use simple_fs::{SPath, ensure_dir};
use std::fs::File;
use std::io::{Write, copy};
use std::path::Path;
use std::time::SystemTime;
use time::OffsetDateTime;

// region:    --- PackUri

enum PackUri {
	LocalPath(String),
	HttpLink(String),
}

impl PackUri {
	fn parse(uri: &str) -> Self {
		if uri.starts_with("http://") || uri.starts_with("https://") {
			PackUri::HttpLink(uri.to_string())
		} else {
			PackUri::LocalPath(uri.to_string())
		}
	}
}

// endregion: --- PackUri

pub struct InstalledPack {
	pub pack_toml: PackToml,
	pub path: SPath,
	pub size: usize,
	pub zip_size: usize,
}

/// Install a `file.aipack` into the .aipack-base/pack/installed directory
///
/// IMPORTANT: Right now, very prelimealy. Should do the following:
///
/// TODO:
/// - Check for an existing installed pack.
/// - If an already installed pack has a semver greater than the new one,
///   return an error so that the caller can handle it with a prompt, and then provide a force flag, for example.
/// - Probably need to remove the existing pack files; otherwise, some leftover files can be an issue.
///
/// Returns the InstalledPack with information about the installed pack.
pub async fn install_pack(dir_context: &DirContext, pack_uri: &str) -> Result<InstalledPack> {
	let pack_uri = PackUri::parse(pack_uri);

	// Get the aipack file path, downloading if needed
	let aipack_zipped_file = match pack_uri {
		PackUri::LocalPath(path) => resolve_local_path(dir_context, &path)?,
		PackUri::HttpLink(url) => download_pack(dir_context, &url).await?,
	};

	// Validate file exists and has correct extension
	validate_aipack_file(&aipack_zipped_file)?;

	// Get the zip file size
	let zip_size = get_file_size(&aipack_zipped_file)?;

	// Common installation steps for both local and remote files
	let mut installed_pack = install_aipack_file(dir_context, &aipack_zipped_file)?;
	installed_pack.zip_size = zip_size;

	Ok(installed_pack)
}

/// Resolves a local path to an absolute SPath
fn resolve_local_path(dir_context: &DirContext, path: &str) -> Result<SPath> {
	let aipack_zipped_file = SPath::from(path);

	if aipack_zipped_file.path().is_absolute() {
		Ok(aipack_zipped_file)
	} else {
		Ok(dir_context.current_dir().join_str(aipack_zipped_file.to_str()))
	}
}

/// Downloads a pack from a URL and returns the path to the downloaded file
async fn download_pack(dir_context: &DirContext, url: &str) -> Result<SPath> {
	// Get the download directory
	let download_dir = dir_context.aipack_paths().get_base_pack_download_dir()?;

	// Create the download directory if it doesn't exist
	if !download_dir.exists() {
		ensure_dir(&download_dir)?;
	}

	// Extract the filename from the URL
	let url_path = url.split('/').last().unwrap_or("unknown.aipack");
	let filename = url_path.replace(' ', "-");

	// Create a timestamped filename using the time crate
	let now = OffsetDateTime::now_utc();
	let timestamp = now
		.format(&time::format_description::well_known::Rfc3339)
		.map_err(|e| Error::FailToInstall {
			aipack_file: url.to_string(),
			cause: format!("Failed to format timestamp: {}", e),
		})?;

	// Create a cleaner timestamp for filenames (removing colons, etc.)
	let file_timestamp = timestamp.replace([':', 'T'], "-");
	let file_timestamp = file_timestamp.split('.').next().unwrap_or(timestamp.as_str());
	let timestamped_filename = format!("{}-{}", file_timestamp, filename);
	let download_path = download_dir.join_str(&timestamped_filename);

	// Download the file
	let client = Client::new();
	let response = client.get(url).send().await.map_err(|e| Error::FailToInstall {
		aipack_file: url.to_string(),
		cause: format!("Failed to download file: {}", e),
	})?;

	// Check if the request was successful
	if !response.status().is_success() {
		return Err(Error::FailToInstall {
			aipack_file: url.to_string(),
			cause: format!("HTTP error: {}", response.status()),
		});
	}

	// Create the output file
	let mut file = File::create(download_path.path()).map_err(|e| Error::FailToInstall {
		aipack_file: url.to_string(),
		cause: format!("Failed to create file: {}", e),
	})?;

	// Stream the response body to file
	let mut stream = response.bytes_stream();
	use tokio::fs::File as TokioFile;
	use tokio::io::AsyncWriteExt;

	// We need to use tokio's async file for proper streaming
	let mut file = TokioFile::create(download_path.path())
		.await
		.map_err(|e| Error::FailToInstall {
			aipack_file: url.to_string(),
			cause: format!("Failed to create file: {}", e),
		})?;

	while let Some(chunk_result) = tokio_stream::StreamExt::next(&mut stream).await {
		let chunk = chunk_result.map_err(|e| Error::FailToInstall {
			aipack_file: url.to_string(),
			cause: format!("Failed to download chunk: {}", e),
		})?;

		file.write_all(&chunk).await.map_err(|e| Error::FailToInstall {
			aipack_file: url.to_string(),
			cause: format!("Failed to write chunk to file: {}", e),
		})?;
	}

	file.flush().await.map_err(|e| Error::FailToInstall {
		aipack_file: url.to_string(),
		cause: format!("Failed to flush file: {}", e),
	})?;

	Ok(download_path)
}

/// Validates that the file exists and has the correct extension
fn validate_aipack_file(aipack_zipped_file: &SPath) -> Result<()> {
	if !aipack_zipped_file.exists() {
		return Err(Error::FailToInstall {
			aipack_file: aipack_zipped_file.to_string(),
			cause: "aipack file does not exist".to_string(),
		});
	}

	if aipack_zipped_file.ext() != "aipack" {
		return Err(Error::FailToInstall {
			aipack_file: aipack_zipped_file.to_string(),
			cause: format!(
				"aipack file must be '.aipack' file, but was {}",
				aipack_zipped_file.name()
			),
		});
	}

	Ok(())
}

/// Get the size of a file in bytes
fn get_file_size(file_path: &SPath) -> Result<usize> {
	let metadata = std::fs::metadata(file_path.path()).map_err(|e| Error::FailToInstall {
		aipack_file: file_path.to_string(),
		cause: format!("Failed to get file metadata: {}", e),
	})?;

	Ok(metadata.len() as usize)
}

/// Common installation logic for both local and remote aipack files
/// Return the InstalledPack containing pack information and installation details
fn install_aipack_file(dir_context: &DirContext, aipack_zipped_file: &SPath) -> Result<InstalledPack> {
	// -- Get the aipack base pack install dir
	// This is the pack base dir and now, we need ot add `namespace/pack_name`
	let pack_installed_dir = dir_context.aipack_paths().get_base_pack_installed_dir()?;

	if !pack_installed_dir.exists() {
		return Err(Error::FailToInstall {
			aipack_file: aipack_zipped_file.to_string(),
			cause: format!(
				"aipack base directory '{pack_installed_dir}' not found.\n   recommendation: Run 'aip init'"
			),
		});
	}

	// -- Extract the pack.toml from zip
	let toml_content = zip::extract_text_content(aipack_zipped_file, "pack.toml")?;
	let pack_toml = parse_validate_pack_toml(&toml_content, &format!("{aipack_zipped_file} pack.toml"))?;

	let pack_target_dir = pack_installed_dir.join_str(&pack_toml.namespace).join_str(&pack_toml.name);

	zip::unzip_file(aipack_zipped_file, &pack_target_dir)?;

	// Calculate the size of the installed pack
	let size = calculate_directory_size(&pack_target_dir)?;

	Ok(InstalledPack {
		pack_toml,
		path: pack_target_dir,
		size,
		zip_size: 0, // This will be populated by the caller
	})
}

/// Calculate the total size of a directory recursively
fn calculate_directory_size(dir_path: &SPath) -> Result<usize> {
	use walkdir::WalkDir;

	let total_size = WalkDir::new(dir_path.path())
		.into_iter()
		.filter_map(|entry| entry.ok())
		.filter_map(|entry| entry.metadata().ok())
		.filter(|metadata| metadata.is_file())
		.map(|metadata| metadata.len() as usize)
		.sum();

	Ok(total_size)
}

// region:    --- Tests

// TODO: Need to write basic test about this one

// endregion: --- Tests
