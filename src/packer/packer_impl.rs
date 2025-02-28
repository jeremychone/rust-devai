//! Module that pack the files into their .aipack

use crate::pack::PackIdentity;
use crate::packer::pack_toml::{PackDirData, PackToml, PartialPackToml};
use crate::support::zip;
use crate::{Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
use lazy_regex::regex;
use serde::Deserialize;
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
	let pack_toml = validate_pack_toml(&toml_content, &toml_path)?;

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
	zip::zip_dir(pack_dir, &aipack_path).map_err(|e| Error::Zip(e.to_string()))?;

	Ok(PackDirData {
		pack_file: aipack_path.into(),
		pack_toml,
	})
}

/// Validates the pack.toml content and returns a PackToml struct if valid
///
/// # Parameters
/// - `toml_content`: The content of the pack.toml file
/// - `toml_path`: The path to the pack.toml file (for error reporting)
///
/// # Returns
/// - Ok(PackToml): If validation is successful
/// - Err(Error): If any required field is missing, empty, or invalid
fn validate_pack_toml(toml_content: &str, toml_path: &Utf8Path) -> Result<PackToml> {
	// Parse the TOML content
	let partial_config: PartialPackToml = toml::from_str(toml_content)?;

	// Validate fields
	let version = match &partial_config.version {
		Some(v) if !v.trim().is_empty() => v.clone(),
		_ => return Err(Error::VersionMissing(toml_path.to_string())),
	};

	// Validate version format
	validate_version(&version, toml_path)?;

	let namespace = match &partial_config.namespace {
		Some(n) if !n.trim().is_empty() => n.clone(),
		_ => return Err(Error::NamespaceMissing(toml_path.to_string())),
	};

	let name = match &partial_config.name {
		Some(n) if !n.trim().is_empty() => n.clone(),
		_ => return Err(Error::NameMissing(toml_path.to_string())),
	};

	// Validate namespace and name format
	validate_names(&namespace, &name, toml_path)?;

	Ok(PackToml {
		version,
		namespace,
		name,
	})
}

/// Validates the version string according to semver compatibility
///
/// Version must follow the format x.y.z and can optionally have a -suffix.number
fn validate_version(version: &str, toml_path: &Utf8Path) -> Result<()> {
	// Using lazy-regex to create a regex for semver format
	let re = regex!(r"^(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z][\w-]*)(?:\.(\d+))?)?$");

	if !re.is_match(version) {
		return Err(Error::custom(format!(
			"Invalid version format in {}. Version must follow semver format (e.g., 1.0.0, 1.0.0-alpha.1)",
			toml_path
		)));
	}

	Ok(())
}

/// Validates namespace and package name
///
/// Names can only contain alphanumeric characters, hyphens, and underscores,
/// and cannot start with a number
fn validate_names(namespace: &str, name: &str, toml_path: &Utf8Path) -> Result<()> {
	PackIdentity::validate_namespace(namespace)?;
	PackIdentity::validate_name(name)?;
	Ok(())
}

/// Normalizes a version string by replacing dots and special characters with hyphens
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

#[cfg(test)]
mod tests {
	use super::*;
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_pack_normalize_version_simple() {
		assert_eq!(normalize_version("1.0.0"), "1-0-0");
		assert_eq!(normalize_version("1.0-alpha"), "1-0-alpha");
		assert_eq!(normalize_version("1.0 beta"), "1-0-beta");
		assert_eq!(normalize_version("1.0-beta-2"), "1-0-beta-2");
		assert_eq!(normalize_version("1.0--beta--2"), "1-0-beta-2");
		assert_eq!(normalize_version("v1.0.0_rc1"), "v1-0-0-rc1");
		assert_eq!(normalize_version("1.0.0!@#$%^&*()"), "1-0-0");
	}

	#[test]
	fn test_pack_validate_pack_toml_simple() -> Result<()> {
		// -- Setup & Fixtures
		let valid_toml = r#"
version = "1.0.0"
namespace = "test"
name = "pack"
"#;
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec
		let pack_toml = validate_pack_toml(valid_toml, &toml_path)?;

		// -- Check
		assert_eq!(pack_toml.version, "1.0.0");
		assert_eq!(pack_toml.namespace, "test");
		assert_eq!(pack_toml.name, "pack");

		Ok(())
	}

	#[test]
	fn test_pack_validate_pack_toml_missing_fields() -> Result<()> {
		// -- Setup & Fixtures
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec & Check
		// Missing version
		let toml_missing_version = r#"
namespace = "test"
name = "pack"
"#;
		let result = validate_pack_toml(toml_missing_version, &toml_path);
		assert!(matches!(result, Err(Error::VersionMissing(_))));

		// Missing namespace
		let toml_missing_namespace = r#"
version = "1.0.0"
name = "pack"
"#;
		let result = validate_pack_toml(toml_missing_namespace, &toml_path);
		assert!(matches!(result, Err(Error::NamespaceMissing(_))));

		// Missing name
		let toml_missing_name = r#"
version = "1.0.0"
namespace = "test"
"#;
		let result = validate_pack_toml(toml_missing_name, &toml_path);
		assert!(matches!(result, Err(Error::NameMissing(_))));

		// Empty values
		let toml_empty_values = r#"
version = ""
namespace = "test"
name = "pack"
"#;
		let result = validate_pack_toml(toml_empty_values, &toml_path);
		assert!(matches!(result, Err(Error::VersionMissing(_))));

		Ok(())
	}

	#[test]
	fn test_pack_validate_version() -> Result<()> {
		// -- Setup & Fixtures
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec & Check
		// Valid versions
		assert!(validate_version("1.0.0", &toml_path).is_ok());
		assert!(validate_version("0.1.0", &toml_path).is_ok());
		assert!(validate_version("10.20.30", &toml_path).is_ok());
		assert!(validate_version("1.0.0-alpha", &toml_path).is_ok());
		assert!(validate_version("1.0.0-alpha.1", &toml_path).is_ok());
		assert!(validate_version("1.0.0-beta-rc.1", &toml_path).is_ok());

		// Invalid versions
		assert!(validate_version("1.0", &toml_path).is_err());
		assert!(validate_version("1", &toml_path).is_err());
		assert!(validate_version("1.0.0.0", &toml_path).is_err());
		assert!(validate_version("1.0.0-", &toml_path).is_err());
		assert!(validate_version("1.0.0-alpha.beta", &toml_path).is_err());
		assert!(validate_version("1.0.0-alpha.1.2", &toml_path).is_err());
		assert!(validate_version("1.0.0-1alpha", &toml_path).is_err());
		assert!(validate_version("a.b.c", &toml_path).is_err());

		Ok(())
	}

	#[test]
	fn test_pack_validate_names() -> Result<()> {
		// -- Setup & Fixtures
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec & Check
		// Valid names
		assert!(validate_names("test", "pack", &toml_path).is_ok());
		assert!(validate_names("my_namespace", "my_package", &toml_path).is_ok());
		assert!(validate_names("my-namespace", "my-package", &toml_path).is_ok());
		assert!(validate_names("myNamespace", "myPackage", &toml_path).is_ok());
		assert!(validate_names("_test", "_pack", &toml_path).is_ok());
		assert!(validate_names("test123", "pack456", &toml_path).is_ok());

		// Invalid names
		assert!(validate_names("1test", "pack", &toml_path).is_err());
		assert!(validate_names("test", "1pack", &toml_path).is_err());
		assert!(validate_names("test!", "pack", &toml_path).is_err());
		assert!(validate_names("test", "pack!", &toml_path).is_err());
		assert!(validate_names("test.space", "pack", &toml_path).is_err());
		assert!(validate_names("test", "pack.name", &toml_path).is_err());
		assert!(validate_names("test/space", "pack", &toml_path).is_err());
		assert!(validate_names("test", "pack/name", &toml_path).is_err());

		Ok(())
	}
}
