use simple_fs::{ensure_file_dir, SPath};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;

const INIT_ASSETS_PATH: &str = ".assets/init_assets.zip";

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let zip_path = SPath::from(INIT_ASSETS_PATH);
	ensure_file_dir(&zip_path)?;
	// Determine output directory and zip file path
	let zip_file = File::create(zip_path)?;
	let mut zip = zip::ZipWriter::new(zip_file);
	let options = FileOptions::default()
		.compression_method(CompressionMethod::Deflated)
		.unix_permissions(0o755);
	let init_dir = Path::new("_init");
	let mut buffer = Vec::new();

	// Recursively add files from the _init folder
	for entry in WalkDir::new(init_dir) {
		let entry = entry?;
		let path = entry.path();
		let name = path.strip_prefix(init_dir)?.to_str().unwrap();

		if path.is_file() {
			zip.start_file(name, options)?;
			let mut f = File::open(path)?;
			f.read_to_end(&mut buffer)?;
			zip.write_all(&buffer)?;
			buffer.clear();
		} else if !name.is_empty() {
			zip.add_directory(name, options)?;
		}
	}
	zip.finish()?;

	Ok(())
}
