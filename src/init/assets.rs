// Auto-generated file. Do not edit.
use crate::{Error, Result};

pub const ASSETS_ZIP: &[u8] = include_bytes!(env!("INIT_ASSETS_ZIP"));

use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug)]
pub(super) struct ZFile {
	#[allow(unused)]
	pub path: String,
	pub content: Vec<u8>,
}

pub fn extract_workspace_config_toml_zfile() -> Result<ZFile> {
	extract_workspace_zfile("config.toml")
}

pub fn extract_workspace_default_file_paths() -> Result<Vec<String>> {
	list_workspace_file_paths_start_with("default")
}
pub fn extract_workspace_doc_file_paths() -> Result<Vec<String>> {
	list_workspace_file_paths_start_with("doc")
}

pub fn extract_workspace_zfile(path: &str) -> Result<ZFile> {
	let path = format!("workspace/{path}");
	let content = extract_asset_content(&path)?;
	Ok(ZFile {
		path: path.to_string(),
		content,
	})
}

// region:    --- Support

/// List the paths nder the `workspace/_prefix_` path and remove
pub fn list_workspace_file_paths_start_with(prefix: &str) -> Result<Vec<String>> {
	let workspace_dir = "workspace/";

	let archive = new_archive_reader()?;

	let mut paths = Vec::new();

	for path in archive.file_names() {
		if !path.ends_with('/') && path.starts_with(workspace_dir) {
			let Some(path_sub) = path.strip_prefix(workspace_dir) else {
				continue;
			};
			if path_sub.starts_with(prefix) {
				paths.push(path_sub.to_string());
			}
		}
	}

	Ok(paths)
}

fn extract_asset_content(path: &str) -> Result<Vec<u8>> {
	let mut archive = new_archive_reader()?;

	let mut file = archive
		.by_name(path)
		.map_err(|err| Error::custom(format!("Fail to extract assets from zip '{path}'. Cause: {err} ")))?;

	let mut data: Vec<u8> = Vec::new();

	file.read_to_end(&mut data)?;

	Ok(data)
}

fn new_archive_reader() -> Result<ZipArchive<Cursor<&'static [u8]>>> {
	let reader = Cursor::new(ASSETS_ZIP);

	let archive = ZipArchive::new(reader)
		.map_err(|err| Error::custom(format!("Cannot create zip archive reader. Cause: {err}")))?;

	Ok(archive)
}

// endregion: --- Support
