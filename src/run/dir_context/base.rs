use crate::run::DevaiDir;
use crate::support::files::current_dir;
use crate::Result;
use simple_fs::SPath;
use std::path::Path;

#[allow(clippy::enum_variant_names)] // to remove
pub enum PathResolver {
	CurrentDir,
	DevaiParentDir,
	DevaiDir,
}

#[derive(Debug, Clone)]
pub struct DirContext {
	/// Absolute path of the current_dir (pwd)
	/// (except for test, which can be mocked to another dir)
	current_dir: SPath,

	devai_dir: DevaiDir,

	// Absolute path of the devai
	workspace_dir: SPath,
}

/// Constructor/Loader
impl DirContext {
	pub fn new(devai_dir: DevaiDir) -> Result<Self> {
		let current_dir = current_dir()?;
		Self::from_devai_dir_and_current_dir(devai_dir, current_dir)
	}

	/// Private to create a new DirContext
	/// Note: Only the test function will provide a mock current_dir
	fn from_devai_dir_and_current_dir(devai_dir: DevaiDir, current_dir: SPath) -> Result<Self> {
		let workspace_dir = devai_dir.parent_dir().canonicalize()?;
		let current_dir = current_dir.canonicalize()?;
		Ok(Self {
			current_dir,
			devai_dir,
			workspace_dir,
		})
	}

	/// Here is a test function that create a new DirContext with a Mock current dir
	#[cfg(test)]
	pub fn from_parent_dir_and_current_dir_for_test(
		parent_dir: impl AsRef<std::path::Path>,
		mock_current_dir: SPath,
	) -> Result<Self> {
		Self::from_devai_dir_and_current_dir(DevaiDir::from_parent_dir(parent_dir)?, mock_current_dir)
	}
}

/// Property Getters
impl DirContext {
	pub fn current_dir(&self) -> &SPath {
		&self.current_dir
	}

	/// Will always be `"./.devai/"`
	pub fn devai_dir(&self) -> &DevaiDir {
		&self.devai_dir
	}

	pub fn workspace_dir(&self) -> &SPath {
		&self.workspace_dir
	}
}

/// Resolvers
impl DirContext {
	pub fn resolve_path(&self, path: impl AsRef<Path>, mode: PathResolver) -> Result<SPath> {
		let path = SPath::from_path(path)?;

		if path.path().is_absolute() {
			Ok(path)
		} else {
			match mode {
				PathResolver::CurrentDir => Ok(self.current_dir.join(path)?),
				PathResolver::DevaiParentDir => Ok(self.workspace_dir.join(path)?),
				PathResolver::DevaiDir => Ok(self.devai_dir().devai_dir_full_path().join(path)?),
			}
		}
	}
}
