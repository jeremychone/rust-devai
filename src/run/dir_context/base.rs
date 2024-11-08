use crate::run::DevaiDir;
use crate::support::files::current_dir;
use crate::{Error, Result};
use simple_fs::SPath;
use std::fs::canonicalize;

#[derive(Debug, Clone)]
pub struct DirContext {
	/// Absolute path of the current_dir (pwd)
	current_dir: SPath,

	devai_dir: DevaiDir,

	// Absolute path of the devai
	devai_parent_dir: SPath,
}

/// Constructor/Loader
impl DirContext {
	pub fn new(devai_dir: DevaiDir) -> Result<Self> {
		let current_dir = current_dir()?;
		let devai_parent_dir = devai_dir.parent_dir().canonicalize()?;
		Ok(Self {
			current_dir,
			devai_dir,
			devai_parent_dir,
		})
	}

	#[cfg(test)]
	pub fn from_parent_dir(parent_dir: impl AsRef<std::path::Path>) -> Result<Self> {
		let parent_dir = SPath::new(parent_dir.as_ref())?;
		let devai_dir = DevaiDir::from_parent_dir(&parent_dir)?;
		Self::new(devai_dir)
	}
}

/// Property Getters
impl DirContext {
	pub fn current_dir(&self) -> &SPath {
		&self.current_dir
	}

	pub fn devai_dir(&self) -> &DevaiDir {
		&self.devai_dir
	}

	pub fn devai_parent_dir(&self) -> &SPath {
		&self.devai_parent_dir
	}
}
