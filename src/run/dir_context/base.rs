use crate::run::DevaiDir;
use crate::support::files::current_dir;
use crate::{Error, Result};
use simple_fs::SPath;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DirContext {
	/// Absolute path of the current_dir (pwd)
	current_dir: SPath,

	/// Absolute path of the devai_dir
	devai_dir: DevaiDir,

	/// Absolute path of the parent of the devai_dir
	devai_parent_dir: SPath,

	/// Absolute path of were to run the file from
	/// Typically teh parent dir of the devai_dir
	ref_dir: SPath,
}

/// Constructor/Loader
impl DirContext {
	pub fn load() -> Result<Option<Self>> {
		let current_dir = current_dir()?;

		// -- find .devai folder
		if let Some((devai_parent_dir, devai_dir)) = find_devai_parent_and_dir(&current_dir)? {
			// for now, ref_dir is current_dir
			let ref_dir = current_dir.clone();

			Ok(Some(Self {
				current_dir,
				devai_dir,
				devai_parent_dir,
				ref_dir,
			}))
		} else {
			Ok(None)
		}
	}

	#[cfg(test)]
	pub fn from_parent_dir(parent_dir: impl AsRef<Path>) -> Result<Self> {
		let parent_dir = SPath::new(parent_dir.as_ref())?;
		let devai_dir = DevaiDir::from_parent_dir(&parent_dir)?;

		Ok(Self {
			current_dir: current_dir()?,
			devai_dir,
			devai_parent_dir: parent_dir,
			ref_dir: current_dir()?,
		})
	}
}

/// Property Getters
#[allow(unused)]
impl DirContext {
	pub fn pwd(&self) -> &SPath {
		&self.current_dir
	}

	pub fn devai_dir(&self) -> &DevaiDir {
		&self.devai_dir
	}

	pub fn devai_parent_dir(&self) -> &SPath {
		&self.devai_parent_dir
	}

	pub fn ref_dir(&self) -> &SPath {
		&self.ref_dir
	}
}

// region:    --- Support

/// Return an option of spath tuple as (devai_parent_dir, devai_dir)
fn find_devai_parent_and_dir(from_dir: &SPath) -> Result<Option<(SPath, DevaiDir)>> {
	let mut tmp_dir: Option<PathBuf> = Some(from_dir.into());

	while let Some(parent_dir) = tmp_dir {
		let devai_dir = DevaiDir::from_parent_dir(&parent_dir)?;

		if devai_dir.exists() {
			return Ok(Some((SPath::new(parent_dir)?, devai_dir)));
		}

		tmp_dir = parent_dir.parent().map(|p| p.into());
	}

	Ok(None)
}

// endregion: --- Support
