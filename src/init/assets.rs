// Auto-generated file. Do not edit.

pub const ASSETS_ZIP: &[u8] = include_bytes!("../../.assets/init_assets.zip");

use std::io::{Cursor, Read};
use zip::ZipArchive;

pub fn list_assets() -> Result<Vec<String>, Box<dyn std::error::Error>> {
	let reader = Cursor::new(ASSETS_ZIP);
	let mut archive = ZipArchive::new(reader)?;
	let mut files = Vec::new();
	for i in 0..archive.len() {
		let file = archive.by_index(i)?;
		files.push(file.name().to_string());
	}
	Ok(files)
}

pub fn extract_asset(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
	let reader = Cursor::new(ASSETS_ZIP);
	let mut archive = ZipArchive::new(reader)?;
	let mut file = archive.by_name(path)?;
	let mut data = Vec::new();
	file.read_to_end(&mut data)?;
	Ok(data)
}
