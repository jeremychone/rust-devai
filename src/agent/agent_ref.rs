//! This module defines the AgentRef enum used to reference an agent either as a local file path
//! or as a PackRef, which can be parsed from a string using the '@' delimiter.

use crate::run::{PackDir, RepoKind};
use simple_fs::SPath;

/// AgentRef represents a reference to an agent.
/// It can either be a LocalPath (a direct file path) or a PackRef (a parsed representation).
#[derive(Debug, Clone)]
pub enum PartialAgentRef {
	LocalPath(String),
	PackRef(PartialPackRef),
}

/// Constructors
impl PartialAgentRef {
	/// Create a new AgentRef from an input string.
	///
	/// The function determines the type of AgentRef based on the presence of the '@' character.
	///
	/// If the input string contains '@':
	///   - It is parsed into a PackRef.
	///   - Example: "jc@coder" will be parsed as PackRef {
	///           namespace: Some("jc"),
	///           pack_name: "coder",
	///           sub_path: None
	///       }
	///   - If a subpath is provided (e.g., "jc@coder/explain"), the sub_path field is set.
	///
	/// If the input string does not contain '@':
	///   - It is treated as a local file path.
	pub fn new(input: &str) -> Self {
		// Check if the input contains the delimiter '@'
		if input.contains('@') {
			// Split the input into namespace and remainder using '@'
			let parts: Vec<&str> = input.splitn(2, '@').collect();
			let ns_part = parts[0].trim();
			let remainder = parts[1].trim();

			// Split the remainder into pack_name and an optional sub_path using '/'
			let mut rem_parts = remainder.splitn(2, '/');
			let pack_name = rem_parts.next().unwrap().to_string();
			let sub_path = rem_parts.next().map(|s| s.to_string());

			// Determine if namespace is provided or empty
			let namespace = if ns_part.is_empty() {
				None
			} else {
				Some(ns_part.to_string())
			};

			// Return a PackRef wrapped in the AgentRef enum
			PartialAgentRef::PackRef(PartialPackRef {
				namespace,
				name: pack_name,
				sub_path,
			})
		} else {
			// If no '@' is found, treat the input as a local file path and return it as LocalPath.
			PartialAgentRef::LocalPath(input.to_string())
		}
	}
}

/// Implement the Display trait for AgentRef
impl std::fmt::Display for PartialAgentRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PartialAgentRef::LocalPath(path) => write!(f, "{}", path),
			PartialAgentRef::PackRef(pack_ref) => {
				write!(f, "{}", pack_ref)?;
				Ok(())
			}
		}
	}
}

// region:    --- PackRef

/// PackRef represents a resource reference to a pack resource.
/// For example, a string like "jc@coder/explain" will be parsed into:
///     - namespace: "jc"
///     - pack_name: "coder"
///     - sub_path: Some("explain")
#[derive(Debug, Clone)]
pub struct PartialPackRef {
	pub namespace: Option<String>,
	pub name: String,
	pub sub_path: Option<String>,
}

/// Implement the Display trait for PartialPackRef
impl std::fmt::Display for PartialPackRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(ns) = &self.namespace {
			write!(f, "{}@", ns)?;
		}
		write!(f, "{}", self.name)?;
		if let Some(sub_path) = &self.sub_path {
			write!(f, "/{}", sub_path)?;
		}
		Ok(())
	}
}

// endregion: --- PackRef

// region:    --- Final AgentRef & PackRef

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AgentRef {
	LocalPath(String),
	PackRef(PackRef),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct PackRef {
	pub namespace: String,
	pub name: String,
	/// e.g. `text` if `demo@craft/text`
	pub sub_path: Option<String>,
	/// The absolute path of the pack `demo@craft`
	pub pack_dir: SPath,
	pub repo_kind: RepoKind,
}

impl PackRef {
	/// NOTE: Right now ns and pack_name ae in both pack_dir and partial, but that is ok for no
	///       Eventually, need to clean this up.
	pub fn from_partial(pack_dir: PackDir, partial: PartialPackRef) -> Self {
		let repo_kind = pack_dir.repo_kind;
		let namespace = pack_dir.namespace;
		let pack_dir = pack_dir.path;

		Self {
			namespace,
			name: partial.name,
			sub_path: partial.sub_path,
			pack_dir,
			repo_kind,
		}
	}
}

/// Implement the Display trait for PackRef
impl std::fmt::Display for PackRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.namespace, self.name)?;
		if let Some(sub_path) = &self.sub_path {
			write!(f, "/{}", sub_path)?;
		}
		Ok(())
	}
}

// endregion: --- Final AgentRef & PackRef

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_agent_ref_new_localpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = "path/to/local/file.rs";

		// -- Exec
		let agent_ref = PartialAgentRef::new(input);

		// -- Check
		match agent_ref {
			PartialAgentRef::LocalPath(ref path) => {
				assert_eq!(path, input, "The local path should match the input string.");
			}
			_ => panic!("Expected AgentRef::LocalPath but got a different variant."),
		}

		Ok(())
	}

	#[test]
	fn test_agent_ref_new_packref_without_subpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = "jc@coder";

		// -- Exec
		let agent_ref = PartialAgentRef::new(input);

		// -- Check
		match agent_ref {
			PartialAgentRef::PackRef(ref pack_ref) => {
				assert_eq!(pack_ref.namespace.as_deref(), Some("jc"), "Namespace should be 'jc'.");
				assert_eq!(pack_ref.name, "coder", "Pack name should be 'coder'.");
				assert!(pack_ref.sub_path.is_none(), "Sub path should be None.");
			}
			_ => panic!("Expected AgentRef::PackRef but got a different variant."),
		}

		Ok(())
	}

	#[test]
	fn test_agent_ref_new_packref_with_subpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = " jc @ coder/example/path ";
		// note: the input contains spaces which should be trimmed

		// -- Exec
		let agent_ref = PartialAgentRef::new(input);

		// -- Check
		match agent_ref {
			PartialAgentRef::PackRef(ref pack_ref) => {
				assert_eq!(pack_ref.namespace.as_deref(), Some("jc"), "Namespace should be 'jc'.");
				assert_eq!(pack_ref.name, "coder", "Pack name should be 'coder'.");
				assert_eq!(
					pack_ref.sub_path.as_deref(),
					Some("example/path"),
					"Sub path should be 'example/path'."
				);
			}
			_ => panic!("Expected AgentRef::PackRef but got a different variant."),
		}

		Ok(())
	}
}

// endregion: --- Tests
