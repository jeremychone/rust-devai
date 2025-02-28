//! Module that pack the files into their .aipack

use crate::packer::pack_toml::{PackDirData, parse_validate_pack_toml};
use crate::support::zip;
use crate::{Error, Result};
use camino::Utf8Path;
use std::fs;

/// Packs a directory into a .aipack file
///
/// # Parameters
/// - `pack_dir`: The directory containing the content to be packed
/// - `dest_dir`: The directory where the .aipack file will be created
///
/// # Returns
/// - Ok(PackDirData): If packing is successful, containing the path to the created .aipack file and pack.toml data
/// - Err(Error): If any error occurs during packing
pub fn pack_dir(pack_dir: impl AsRef<Utf8Path>, dest_dir: impl AsRef<Utf8Path>) -> Result<PackDirData> {
	let pack_dir = pack_dir.as_ref();
	let dest_dir = dest_dir.as_ref();

	// Verify if pack.toml exists
	let toml_path = pack_dir.join("pack.toml");
	if !toml_path.exists() {
		return Err(Error::AipackTomlMissing(toml_path.into()));
	}

	// Read and validate the TOML file
	let toml_content = fs::read_to_string(&toml_path)?;
	let pack_toml = parse_validate_pack_toml(&toml_content, toml_path.as_str())?;

	// Normalize version - replace special characters with hyphens
	let normalized_version = normalize_version(&pack_toml.version);

	// Create the output filename
	let aipack_filename = format!(
		"{}-{}-v-{}.aipack",
		pack_toml.namespace, pack_toml.name, normalized_version
	);
	let aipack_path = dest_dir.join(aipack_filename);

	// Create the destination directory if it doesn't exist
	if !dest_dir.exists() {
		fs::create_dir_all(dest_dir)?;
	}

	// Zip the directory
	zip::zip_dir(pack_dir, &aipack_path)?;

	Ok(PackDirData {
		pack_file: aipack_path.into(),
		pack_toml,
	})
}

/// Normalizes a version string by replacing dots and special characters with hyphens
/// This is just to write the file names (cosmetic)
/// and ensuring no consecutive hyphens
fn normalize_version(version: &str) -> String {
	let mut result = String::new();
	let mut last_was_hyphen = false;

	for c in version.chars() {
		if c.is_alphanumeric() {
			result.push(c);
			last_was_hyphen = false;
		} else if !last_was_hyphen {
			result.push('-');
			last_was_hyphen = true;
		}
	}

	// Remove trailing hyphen if exists
	if result.ends_with('-') {
		result.pop();
	}

	result
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_packer_normalize_version_simple() -> Result<()> {
		assert_eq!(normalize_version("1.0.0"), "1-0-0");
		assert_eq!(normalize_version("1.0-alpha"), "1-0-alpha");
		assert_eq!(normalize_version("1.0 beta"), "1-0-beta");
		assert_eq!(normalize_version("1.0-beta-2"), "1-0-beta-2");
		assert_eq!(normalize_version("1.0--beta--2"), "1-0-beta-2");
		assert_eq!(normalize_version("v1.0.0_rc1"), "v1-0-0-rc1");
		assert_eq!(normalize_version("1.0.0!@#$%^&*()"), "1-0-0");

		Ok(())
	}
}

// endregion: --- Tests
