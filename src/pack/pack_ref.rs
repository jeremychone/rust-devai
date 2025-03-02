// region:    --- PartialPackRef

use crate::dir_context::{PackDir, RepoKind};
use crate::pack::PackIdentity;
use simple_fs::SPath;

/// PartialPackRef represents a resource reference to a pack resource.
/// It has not be resolved yet
/// For example, a string like "jc@coder/explain" will be parsed into:
///     - namespace: "jc" (may be omitted)
///     - pack_name: "coder"
///     - sub_path: Some("explain/stuff")
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
			write!(f, "{}@{}", ns, self.name)?;
		} else {
			write!(f, "@{}", self.name)?;
		}
		if let Some(sub_path) = &self.sub_path {
			write!(f, "/{}", sub_path)?;
		}
		Ok(())
	}
}

// endregion: --- PartialPackRef

// region:    --- LocalPackRef

/// This is a Locally Resolved PackRef
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LocalPackRef {
	pub identity: PackIdentity,
	/// e.g. `text` if `demo@craft/text`
	pub sub_path: Option<String>,
	/// The absolute path of the pack `demo@craft`
	pub pack_dir: SPath,
	pub repo_kind: RepoKind,
}

impl LocalPackRef {
	/// NOTE: Right now ns and pack_name ae in both pack_dir and partial, but that is ok for no
	///       Eventually, need to clean this up.
	pub fn from_partial(pack_dir: PackDir, partial: PartialPackRef) -> Self {
		let repo_kind = pack_dir.repo_kind;
		let namespace = pack_dir.namespace;
		let pack_dir = pack_dir.path;

		let identity = PackIdentity {
			namespace,
			name: partial.name,
		};

		Self {
			identity,
			sub_path: partial.sub_path,
			pack_dir,
			repo_kind,
		}
	}
}

/// Getters
#[allow(unused)]
impl LocalPackRef {
	pub fn identity(&self) -> &PackIdentity {
		&self.identity
	}
	pub fn namespace(&self) -> &str {
		&self.identity.namespace
	}
	pub fn name(&self) -> &str {
		&self.identity.name
	}
	pub fn sub_path(&self) -> Option<&str> {
		self.sub_path.as_deref()
	}
	pub fn pack_dir(&self) -> &SPath {
		&self.pack_dir
	}
	pub fn repo_kind(&self) -> RepoKind {
		self.repo_kind
	}
}

/// Implement the Display trait for PackRef
impl std::fmt::Display for LocalPackRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.namespace(), self.name())?;
		if let Some(sub_path) = &self.sub_path {
			write!(f, "/{}", sub_path)?;
		}
		Ok(())
	}
}

// endregion: --- LocalPackRef
