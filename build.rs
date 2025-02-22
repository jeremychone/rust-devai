use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::CompressionMethod;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Get the output directory for build artifacts
	let out_dir = PathBuf::from(env::var("OUT_DIR")?);
	let zip_path = out_dir.join("init_assets.zip");

	// Create zip file
	let zip_file = File::create(&zip_path)?;
	let mut zip = zip::ZipWriter::new(zip_file);
	let options = FileOptions::default()
		.compression_method(CompressionMethod::Deflated)
		.unix_permissions(0o755);
	let init_dir = Path::new("_init");
	let mut buffer = Vec::new();

	// Recursively add files from _init directory
	for entry in WalkDir::new(init_dir) {
		let entry = entry?;
		let path = entry.path();
		let name = path.strip_prefix(init_dir)?.to_str().unwrap();
		let name = name.replace("\\", "/");

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

	// Tell Cargo to watch _init for changes
	println!("cargo:rerun-if-changed=_init");

	// Export the path so it can be used in Rust code
	println!("cargo:rustc-env=INIT_ASSETS_ZIP={}", zip_path.display());

	Ok(())
}
