// Auto-generated file. Do not edit.
use crate::{Error, Result};

pub const ASSETS_ZIP: &[u8] = include_bytes!("../../.assets/init_assets.zip");

use std::io::{Cursor, Read};
use zip::ZipArchive;

pub fn list_assets() -> Result<Vec<String>> {
	let reader = Cursor::new(ASSETS_ZIP);
	let mut archive =
		ZipArchive::new(reader).map_err(|err| Error::custom(format!("Fail to list assets; Cause: {err}")))?;

	let mut files = Vec::new();

	for i in 0..archive.len() {
		let file = archive
			.by_index(i)
			.map_err(|err| Error::custom(format!("Fail to list assets; Cause: {err}")))?;
		files.push(file.name().to_string());
	}

	Ok(files)
}

pub fn extract_asset(path: &str) -> Result<Vec<u8>> {
	let reader = Cursor::new(ASSETS_ZIP);

	let mut archive = ZipArchive::new(reader)
		.map_err(|err| Error::custom(format!("Cannot extract assets for path '{path}'. Cause: {err}")))?;

	let mut file = archive
		.by_name(path)
		.map_err(|err| Error::custom(format!("Fail to extract assets from zip '{path}'. Cause: {err} ")))?;

	let mut data = Vec::new();

	file.read_to_end(&mut data)?;

	Ok(data)
}
