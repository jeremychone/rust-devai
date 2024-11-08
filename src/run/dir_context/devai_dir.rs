use crate::Result;
use simple_fs::SPath;
use std::path::{Path, PathBuf};

// region:    --- Consts

const DEVAI_DIR_NAME: &str = ".devai";

// NOTE: All of the path below are designed to be below the `.devai/` folder

// -- Config
const DEVAI_CONFIG_FILE_PATH: &str = "config.toml";

// -- Command Agent Dirs
const DEVAI_AGENT_DEFAULT_DIR: &str = "default/command-agent";
const DEVAI_AGENT_CUSTOM_DIR: &str = "custom/command-agent";
const DEVAI_COMMAND_AGENT_DIRS: &[&str] = &[
	// by priority
	DEVAI_AGENT_CUSTOM_DIR,
	DEVAI_AGENT_DEFAULT_DIR,
];

// -- New Command Templates
const DEVAI_NEW_CUSTOM_COMMAND_DIR: &str = "custom/new-template/command-agent";
const DEVAI_NEW_DEFAULT_COMMAND_DIR: &str = "default/new-template/command-agent";
const DEVAI_NEW_COMMAND_DIRS: &[&str] = &[
	// by priority
	DEVAI_NEW_CUSTOM_COMMAND_DIR,
	DEVAI_NEW_DEFAULT_COMMAND_DIR,
];

// -- New Solo Templates
const DEVAI_NEW_CUSTOM_SOLO_DIR: &str = "custom/new-template/solo-agent";
const DEVAI_NEW_DEFAULT_SOLO_DIR: &str = "default/new-template/solo-agent";
const DEVAI_NEW_SOLO_DIRS: &[&str] = &[
	// by priority
	DEVAI_NEW_CUSTOM_SOLO_DIR,
	DEVAI_NEW_DEFAULT_SOLO_DIR,
];

// -- Doc
const DEVAI_DOC_DIR: &str = "doc";
const DEVAI_DOC_RHAI_PATH: &str = "doc/rhai.md";

// endregion: --- Consts

#[derive(Debug, Clone)]
pub struct DevaiDir {
	path: SPath,
	parent_dir: SPath,
}

//
impl DevaiDir {
	pub fn from_parent_dir(parent_dir: impl AsRef<Path>) -> Result<Self> {
		let devai_path = parent_dir.as_ref().join(DEVAI_DIR_NAME);
		Self::from_devai_dir(devai_path)
	}

	#[allow(unused)]
	pub fn from_devai_dir(devai_path: impl AsRef<Path>) -> Result<Self> {
		let path = SPath::new(devai_path.as_ref())?;
		let parent_dir = path
			.parent()
			.ok_or_else(|| format!(".devai/ path '{path}' does not have a parent dir (it must have one)"))?;

		Ok(Self { path, parent_dir })
	}
}

/// SPath passthrough
impl DevaiDir {
	pub fn exists(&self) -> bool {
		self.path.exists()
	}

	#[allow(unused)]
	pub fn to_str(&self) -> &str {
		self.path.to_str()
	}

	pub fn parent_dir(&self) -> &SPath {
		&self.parent_dir
	}
}

impl DevaiDir {
	pub fn get_config_toml_path(&self) -> Result<SPath> {
		let path = self.path.join(DEVAI_CONFIG_FILE_PATH)?;
		Ok(path)
	}

	pub fn get_new_template_command_default_dir(&self) -> Result<SPath> {
		let dir = self.path.join(DEVAI_NEW_DEFAULT_COMMAND_DIR)?;
		Ok(dir)
	}

	pub fn get_new_template_command_dirs(&self) -> Result<Vec<SPath>> {
		let dirs = DEVAI_NEW_COMMAND_DIRS
			.iter()
			.map(|&suffix_dir| self.path.join(suffix_dir).map_err(|err| err.into()))
			.collect::<Result<_>>()?;

		Ok(dirs)
	}

	pub fn get_new_template_solo_default_dir(&self) -> Result<SPath> {
		let dir = self.path.join(DEVAI_NEW_DEFAULT_SOLO_DIR)?;
		Ok(dir)
	}

	pub fn get_new_template_solo_dirs(&self) -> Result<Vec<SPath>> {
		let dirs = DEVAI_NEW_SOLO_DIRS
			.iter()
			.map(|&suffix_dir| self.path.join(suffix_dir).map_err(|err| err.into()))
			.collect::<Result<_>>()?;

		Ok(dirs)
	}

	pub fn get_command_agent_dirs(&self) -> Result<Vec<SPath>> {
		let dirs = DEVAI_COMMAND_AGENT_DIRS
			.iter()
			.map(|&suffix_dir| self.path.join(suffix_dir).map_err(|err| err.into()))
			.collect::<Result<_>>()?;

		Ok(dirs)
	}

	pub fn get_command_agent_default_dir(&self) -> Result<SPath> {
		let dir = self.path.join(DEVAI_AGENT_DEFAULT_DIR)?;
		Ok(dir)
	}

	pub fn get_command_agent_custom_dir(&self) -> Result<SPath> {
		let dir = self.path.join(DEVAI_AGENT_CUSTOM_DIR)?;
		Ok(dir)
	}

	pub fn get_doc_dir(&self) -> Result<SPath> {
		let dir = self.path.join(DEVAI_DOC_DIR)?;
		Ok(dir)
	}

	pub fn get_doc_rhai_path(&self) -> Result<SPath> {
		let path = self.path.join(DEVAI_DOC_RHAI_PATH)?;
		Ok(path)
	}
}

// region:    --- Froms & AsRefs

impl AsRef<Path> for DevaiDir {
	fn as_ref(&self) -> &Path {
		self.path.as_ref()
	}
}

// endregion: --- Froms & AsRefs

/// Return an option of spath tuple as (devai_parent_dir, devai_dir)
pub fn find_devai_parent_dir(from_dir: impl AsRef<Path>) -> Result<Option<SPath>> {
	let mut tmp_dir: Option<PathBuf> = Some(from_dir.as_ref().to_path_buf());

	while let Some(parent_dir) = tmp_dir {
		let devai_dir = DevaiDir::from_parent_dir(&parent_dir)?;

		if devai_dir.exists() {
			return Ok(Some(SPath::new(parent_dir)?));
		}

		tmp_dir = parent_dir.parent().map(|p| p.into());
	}

	Ok(None)
}
