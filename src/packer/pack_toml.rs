use crate::pack::PackIdentity;
use crate::{Error, Result};
use lazy_regex::regex;
use serde::Deserialize;
use simple_fs::SPath;

#[derive(Deserialize)]
pub struct PartialPackToml {
	pub version: Option<String>,
	pub namespace: Option<String>,
	pub name: Option<String>,
}

/// Contains the validated required fields from pack.toml
#[derive(Debug, Clone)]
pub struct PackToml {
	pub version: String,
	pub namespace: String,
	pub name: String,
}

/// Data returned when packing a directory
#[derive(Debug)]
pub struct PackDirData {
	pub pack_file: SPath,
	#[allow(unused)]
	pub pack_toml: PackToml,
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
pub(super) fn parse_validate_pack_toml(toml_content: &str, toml_path: &str) -> Result<PackToml> {
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

/// Validates namespace and package name
///
/// Names can only contain alphanumeric characters, hyphens, and underscores,
/// and cannot start with a number
///
/// TODO: Needs to handle the toml_path, it's might be good context to have.
///       Perhaps validate_namespace should take a Option<&str> and use it in case of error
pub(super) fn validate_names(namespace: &str, name: &str, _toml_path: &str) -> Result<()> {
	PackIdentity::validate_namespace(namespace)?;
	PackIdentity::validate_name(name)?;
	Ok(())
}

/// Validates the version string according to semver compatibility
///
/// Version must follow the format x.y.z and can optionally have a -suffix.number
pub(super) fn validate_version(version: &str, toml_path: &str) -> Result<()> {
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

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use camino::Utf8PathBuf;
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_packer_pack_toml_validate_simple() -> Result<()> {
		// -- Setup & Fixtures
		let valid_toml = r#"
version = "1.0.0"
namespace = "test"
name = "pack"
"#;
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec
		let pack_toml = parse_validate_pack_toml(valid_toml, toml_path.as_str())?;

		// -- Check
		assert_eq!(pack_toml.version, "1.0.0");
		assert_eq!(pack_toml.namespace, "test");
		assert_eq!(pack_toml.name, "pack");

		Ok(())
	}

	#[test]
	fn test_packer_pack_toml_validate_missing_fields() -> Result<()> {
		// -- Setup & Fixtures
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec & Check
		// Missing version
		let toml_missing_version = r#"
namespace = "test"
name = "pack"
"#;
		let result = parse_validate_pack_toml(toml_missing_version, toml_path.as_str());
		assert!(matches!(result, Err(Error::VersionMissing(_))));

		// Missing namespace
		let toml_missing_namespace = r#"
version = "1.0.0"
name = "pack"
"#;
		let result = parse_validate_pack_toml(toml_missing_namespace, toml_path.as_str());
		assert!(matches!(result, Err(Error::NamespaceMissing(_))));

		// Missing name
		let toml_missing_name = r#"
version = "1.0.0"
namespace = "test"
"#;
		let result = parse_validate_pack_toml(toml_missing_name, toml_path.as_str());
		assert!(matches!(result, Err(Error::NameMissing(_))));

		// Empty values
		let toml_empty_values = r#"
version = ""
namespace = "test"
name = "pack"
"#;
		let result = parse_validate_pack_toml(toml_empty_values, toml_path.as_str());
		assert!(matches!(result, Err(Error::VersionMissing(_))));

		Ok(())
	}

	#[test]
	fn test_packer_pack_toml_validate_version() -> Result<()> {
		// -- Setup & Fixtures
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec & Check
		// Valid versions
		assert!(validate_version("1.0.0", toml_path.as_str()).is_ok());
		assert!(validate_version("0.1.0", toml_path.as_str()).is_ok());
		assert!(validate_version("10.20.30", toml_path.as_str()).is_ok());
		assert!(validate_version("1.0.0-alpha", toml_path.as_str()).is_ok());
		assert!(validate_version("1.0.0-alpha.1", toml_path.as_str()).is_ok());
		assert!(validate_version("1.0.0-beta-rc.1", toml_path.as_str()).is_ok());

		// Invalid versions
		assert!(validate_version("1.0", toml_path.as_str()).is_err());
		assert!(validate_version("1", toml_path.as_str()).is_err());
		assert!(validate_version("1.0.0.0", toml_path.as_str()).is_err());
		assert!(validate_version("1.0.0-", toml_path.as_str()).is_err());
		assert!(validate_version("1.0.0-alpha.beta", toml_path.as_str()).is_err());
		assert!(validate_version("1.0.0-alpha.1.2", toml_path.as_str()).is_err());
		assert!(validate_version("1.0.0-1alpha", toml_path.as_str()).is_err());
		assert!(validate_version("a.b.c", toml_path.as_str()).is_err());

		Ok(())
	}

	#[test]
	fn test_packer_pack_toml_validate_names() -> Result<()> {
		// -- Setup & Fixtures
		let toml_path = Utf8PathBuf::from("dummy/path/pack.toml");

		// -- Exec & Check
		// Valid names
		assert!(validate_names("test", "pack", toml_path.as_str()).is_ok());
		assert!(validate_names("my_namespace", "my_package", toml_path.as_str()).is_ok());
		assert!(validate_names("my-namespace", "my-package", toml_path.as_str()).is_ok());
		assert!(validate_names("myNamespace", "myPackage", toml_path.as_str()).is_ok());
		assert!(validate_names("_test", "_pack", toml_path.as_str()).is_ok());
		assert!(validate_names("test123", "pack456", toml_path.as_str()).is_ok());

		// Invalid names
		assert!(validate_names("1test", "pack", toml_path.as_str()).is_err());
		assert!(validate_names("test", "1pack", toml_path.as_str()).is_err());
		assert!(validate_names("test!", "pack", toml_path.as_str()).is_err());
		assert!(validate_names("test", "pack!", toml_path.as_str()).is_err());
		assert!(validate_names("test.space", "pack", toml_path.as_str()).is_err());
		assert!(validate_names("test", "pack.name", toml_path.as_str()).is_err());
		assert!(validate_names("test/space", "pack", toml_path.as_str()).is_err());
		assert!(validate_names("test", "pack/name", toml_path.as_str()).is_err());

		Ok(())
	}
}

// endregion: --- Tests
