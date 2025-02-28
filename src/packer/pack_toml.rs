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
