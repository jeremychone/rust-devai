use super::path_consts::PACK_INSTALLED;
use super::path_consts::{AIPACK_BASE, AIPACK_DIR_NAME, CONFIG_FILE_NAME, PACK_CUSTOM};
use crate::dir_context::path_consts::PACK_DOWNLOAD;
use crate::{Error, Result};
use home::home_dir;
use simple_fs::SPath;
use std::path::{Path, PathBuf};

/// AipackPaths is the component that manages all of the Aipack Paths from
/// - workspace paths `./.aipack`
/// - base paths `~/.aipack-base`
///
/// TODO: Might want to explore if we can make the wks optional.
#[derive(Debug, Clone)]
pub struct AipackPaths {
	/// The path to the parent workspace_dir. Can be relative, to working dir for example.
	wks_dir: SPath,

	/// This is absolute path of
	wks_aipack_dir: SPath,

	/// This is absolute path of `~/.aipack-base/`
	base_aipack_dir: SPath,
}

/// Constructor
impl AipackPaths {
	pub fn from_wks_dir(wks_path: impl AsRef<Path>) -> Result<Self> {
		// -- Compute the wks_dir
		let wks_path = wks_path.as_ref();
		if !wks_path.exists() {
			return Err(Error::custom(format!(
				"Cannot run aip, workspace path does not exist {}",
				wks_path.to_string_lossy()
			)));
		}
		let wks_path = wks_path.canonicalize().map_err(|err| {
			Error::custom(format!(
				"Cannot canonicalize wks path for {}: {}",
				wks_path.to_string_lossy(),
				err
			))
		})?;
		let wks_dir = SPath::from_path(wks_path)?;

		// -- Compute the aipack_wks_dir
		// TODO: Needs to define if we have to check if it exists
		//       We want to explore if we can work without workspace, just with the base
		let wks_aipack_dir = wks_dir.join(AIPACK_DIR_NAME)?;

		// -- Compute the aipack_base_dir
		// TODO: Probably need to check that the path exist
		let base_aipack_dir = aipack_base_dir()?;

		Ok(Self {
			wks_dir,
			wks_aipack_dir,
			base_aipack_dir,
		})
	}
}

#[cfg(test)]
impl AipackPaths {
	/// For test use the: DirContext::new_test_runtime_sandbox_01() which will use this to create the mock aipack_paths
	pub fn from_aipack_base_and_wks_dirs(
		base_aipack_dir: impl AsRef<Path>,
		wks_aipack_dir: impl AsRef<Path>,
	) -> Result<Self> {
		let base_aipack_dir = SPath::new(base_aipack_dir.as_ref())?;
		let wks_aipack_dir = SPath::new(wks_aipack_dir.as_ref())?;
		let wks_dir = wks_aipack_dir.parent().ok_or("Should have partent wks_dir (it's for test)")?;
		Ok(AipackPaths {
			wks_dir,
			wks_aipack_dir,
			base_aipack_dir,
		})
	}
}

/// Getters
impl AipackPaths {
	pub fn wks_dir(&self) -> &SPath {
		&self.wks_dir
	}

	pub fn wks_aipack_dir(&self) -> &SPath {
		&self.wks_aipack_dir
	}

	#[allow(unused)]
	pub fn base_aipack_dir(&self) -> &SPath {
		&self.base_aipack_dir
	}
}

#[derive(Debug, Clone, Copy)]
pub enum RepoKind {
	WksCustom,
	BaseCustom,
	BaseInstalled,
}

impl RepoKind {
	pub fn to_pretty_lower(self) -> String {
		match self {
			Self::WksCustom => "workspace custom - .aipack/pack/custom",
			Self::BaseCustom => "base custom - ~/.aipack-base/pack/custom",
			Self::BaseInstalled => "base installed - ~/.aipack-base/pack/installed",
		}
		.to_string()
	}
}

pub struct PackRepo {
	pub kind: RepoKind,
	pub path: SPath,
}

/// Constructor & Getters
impl PackRepo {
	pub fn new(kind: RepoKind, path: SPath) -> Self {
		Self { kind, path }
	}

	#[allow(unused)]
	pub fn to_str(&self) -> &str {
		self.path.to_str()
	}

	pub fn path(&self) -> &SPath {
		&self.path
	}
}

/// Get compute path/s
impl AipackPaths {
	// region:    --- Workspace Files & Dirs
	pub fn get_wks_config_toml_path(&self) -> Result<SPath> {
		let path = self.wks_aipack_dir.join(CONFIG_FILE_NAME)?;
		Ok(path)
	}

	// TOOD: PRobably to return paths of wks, and base
	pub fn get_wks_config_toml_paths(&self) -> Result<Vec<SPath>> {
		let wks_config_path = self.get_wks_config_toml_path()?;
		let base_config_path = self.base_aipack_dir.join(CONFIG_FILE_NAME)?;
		Ok(vec![base_config_path, wks_config_path])
	}

	pub fn get_wks_pack_custom_dir(&self) -> Result<SPath> {
		let dir = self.wks_aipack_dir.join(PACK_CUSTOM)?;
		Ok(dir)
	}
	// endregion: --- Workspace Files & Dirs

	// region:    --- Base Files & Dirs

	pub fn get_base_pack_custom_dir(&self) -> Result<SPath> {
		let dir = self.base_aipack_dir.join(PACK_CUSTOM)?;
		Ok(dir)
	}

	pub fn get_base_pack_installed_dir(&self) -> Result<SPath> {
		let dir = self.base_aipack_dir.join(PACK_INSTALLED)?;
		Ok(dir)
	}

	pub fn get_base_pack_download_dir(&self) -> Result<SPath> {
		let dir = self.base_aipack_dir.join(PACK_DOWNLOAD)?;
		Ok(dir)
	}

	// endregion: --- Base Files & Dirs

	/// Returns the list of pack dirs, in the order of precedence.
	///
	/// The array will contain:
	/// - `/path/to/wks/.aipack/pack/custom`
	/// - `/path/user/home/.aipack-base/pack/custom`
	/// - `/path/user/home/.aipack-base/pack/installed`
	pub fn get_pack_repo_dirs(&self) -> Result<Vec<PackRepo>> {
		let mut dirs = Vec::new();

		// 1. Workspace custom directory: .aipack/pack/custom
		let wks_custom = self.get_wks_pack_custom_dir()?;
		if wks_custom.exists() {
			dirs.push(PackRepo::new(RepoKind::WksCustom, wks_custom));
		}

		// 2. Base custom directory: ~/.aipack-base/pack/custom
		let base_custom = self.get_base_pack_custom_dir()?;
		if base_custom.exists() {
			dirs.push(PackRepo::new(RepoKind::BaseCustom, base_custom));
		}

		// 3. Base installed directory: ~/.aipack-base/pack/installed
		let base_installed = self.get_base_pack_installed_dir()?;
		if base_installed.exists() {
			dirs.push(PackRepo::new(RepoKind::BaseInstalled, base_installed));
		}

		Ok(dirs)
	}
}

/// This returns the `~/.aipack-base` full path
///
/// NOTE: This does NOT create or test if the path exists
///
pub fn aipack_base_dir() -> Result<SPath> {
	let home_dir = home_dir().ok_or("No Home Dir Found, cannot init ./aipack-base")?;
	if !home_dir.exists() {
		Err(format!("Home dir '{}' does not exist", home_dir.to_string_lossy()))?;
	}

	let base_dir = SPath::new(home_dir.join(AIPACK_BASE))?;

	Ok(base_dir)
}

/// Return an option of spath tuple as (workspace_dir, aipack_dir)
pub fn find_wks_dir(from_dir: impl AsRef<Path>) -> Result<Option<SPath>> {
	let mut tmp_dir: Option<PathBuf> = Some(from_dir.as_ref().to_path_buf());

	while let Some(parent_dir) = tmp_dir {
		let wks_dir = AipackPaths::from_wks_dir(&parent_dir)?;

		if wks_dir.wks_aipack_dir().exists() {
			return Ok(Some(SPath::new(parent_dir)?));
		}

		tmp_dir = parent_dir.parent().map(|p| p.into());
	}

	Ok(None)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{SANDBOX_01_WKS_DIR, assert_ends_with};
	use crate::run::Runtime;

	#[test]
	fn test_aipack_dir_simple() -> Result<()> {
		// -- Exec
		let aipack_paths = AipackPaths::from_wks_dir(SANDBOX_01_WKS_DIR)?;

		// -- Check
		// check longer, because shouuld be absolute path
		assert_ends_with(
			aipack_paths.get_wks_config_toml_paths()?[1].to_str(),
			"tests-data/sandbox-01/.aipack/config.toml",
		);

		Ok(())
	}

	#[test]
	fn test_get_pack_dirs() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let aipack_paths = runtime.dir_context().aipack_paths();

		// -- Exec
		let dirs = aipack_paths.get_pack_repo_dirs()?;

		// -- Check
		assert_eq!(dirs.len(), 3);
		assert_ends_with(dirs[0].to_str(), ".aipack/pack/custom");
		assert_ends_with(dirs[1].to_str(), ".aipack-base/pack/custom");
		assert_ends_with(dirs[2].to_str(), ".aipack-base/pack/installed");

		Ok(())
	}
}

// endregion: --- Tests
