use super::AipackPaths;
use crate::Result;
use crate::support::files::current_dir;
use simple_fs::SPath;
use std::path::Path;

#[allow(clippy::enum_variant_names)] // to remove
pub enum PathResolver {
	CurrentDir,
	WksDir,
	#[allow(unused)]
	AipackDir,
}

#[derive(Debug, Clone)]
pub struct DirContext {
	/// Absolute path of the current_dir (pwd)
	/// (except for test, which can be mocked to another dir)
	current_dir: SPath,

	/// This is workspace `.aipack/`
	aipack_paths: AipackPaths,
}

/// Constructor/Loader
impl DirContext {
	pub fn new(aipack_dir: AipackPaths) -> Result<Self> {
		let current_dir = current_dir()?;
		Self::from_aipack_dir_and_current_dir(aipack_dir, current_dir)
	}

	/// Private to create a new DirContext
	/// Note: Only the test function will provide a mock current_dir
	fn from_aipack_dir_and_current_dir(aipack_dir: AipackPaths, current_dir: SPath) -> Result<Self> {
		let current_dir = current_dir.canonicalize()?;
		Ok(Self {
			current_dir,
			aipack_paths: aipack_dir,
		})
	}

	#[cfg(test)]
	pub fn from_current_and_aipack_paths(current_dir: SPath, aipack_paths: AipackPaths) -> Result<Self> {
		Ok(Self {
			current_dir,
			aipack_paths,
		})
	}
}

/// Property Getters
impl DirContext {
	pub fn current_dir(&self) -> &SPath {
		&self.current_dir
	}

	/// Will always be `"./.aipack/"`
	pub fn aipack_paths(&self) -> &AipackPaths {
		&self.aipack_paths
	}

	pub fn wks_dir(&self) -> &SPath {
		self.aipack_paths().wks_dir()
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
				PathResolver::WksDir => Ok(self.wks_dir().join(path)?),
				PathResolver::AipackDir => Ok(self.aipack_paths().wks_aipack_dir().join(path)?),
			}
		}
	}
}
