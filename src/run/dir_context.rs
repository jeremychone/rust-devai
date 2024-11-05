use crate::support::files::current_dir;
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
	DEVAI_AGENT_CUSTOM_DIR,
];

// -- New Command Templates
const DEVAI_NEW_CUSTOM_COMMAND_AGENT_DIR: &str = "custom/new-template/command-agent";
const DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR: &str = "default/new-template/command-agent";
const DEVAI_NEW_COMMAND_DIRS: &[&str] = &[
	// by priority
	DEVAI_NEW_CUSTOM_COMMAND_AGENT_DIR,
	DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR,
];

// -- New Solo Templates
const DEVAI_NEW_CUSTOM_SOLO_AGENT_DIR: &str = "custom/new-template/solo-agent";
const DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR: &str = "default/new-template/solo-agent";
const DEVAI_NEW_SOLO_DIRS: &[&str] = &[
	// by priority
	DEVAI_NEW_CUSTOM_SOLO_AGENT_DIR,
	DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR,
];

// -- Doc
const DEVAI_DOC_DIR: &str = "doc";
const DEVAI_DOC_RHAI_PATH: &str = "doc/rhai.md";

// endregion: --- Consts

#[allow(unused)]
pub struct DirContext {
	/// Absolute path of the current_dir (pwd)
	current_dir: SPath,

	/// Absolute path of the devai_dir
	devai_dir: SPath,

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
}

/// Property Getters
#[allow(unused)]
impl DirContext {
	pub fn pwd(&self) -> &SPath {
		&self.current_dir
	}

	pub fn devai_dir(&self) -> &SPath {
		&self.devai_dir
	}

	pub fn devai_parent_dir(&self) -> &SPath {
		&self.devai_parent_dir
	}

	pub fn ref_dir(&self) -> &SPath {
		&self.ref_dir
	}
}

/// Dirs List
impl DirContext {
	pub fn get_devai_dir(parent_dir: impl AsRef<Path>) -> Result<SPath> {
		let dir = SPath::new(parent_dir.as_ref().join(DEVAI_DIR_NAME))?;
		Ok(dir)
	}

	pub fn get_config_toml_path(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let path = devai_dir.as_ref().join(DEVAI_CONFIG_FILE_PATH);
		let path = SPath::new(path)?;
		Ok(path)
	}

	pub fn get_new_template_command_default_dir(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let dir = devai_dir.as_ref().join(DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR);
		let dir = SPath::new(dir)?;
		Ok(dir)
	}

	pub fn get_new_template_command_dirs(devai_dir: impl AsRef<Path>) -> Result<Vec<SPath>> {
		let devai_dir = devai_dir.as_ref();
		let dirs = DEVAI_NEW_COMMAND_DIRS
			.iter()
			.map(|&suffix_dir| SPath::new(devai_dir.join(suffix_dir)).map_err(|err| err.into()))
			.collect::<Result<_>>()?;

		Ok(dirs)
	}

	pub fn get_new_template_solo_default_dir(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let dir = devai_dir.as_ref().join(DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR);
		let dir = SPath::new(dir)?;
		Ok(dir)
	}

	pub fn get_new_template_solo_dirs(devai_dir: impl AsRef<Path>) -> Result<Vec<SPath>> {
		let devai_dir = devai_dir.as_ref();
		let dirs = DEVAI_NEW_SOLO_DIRS
			.iter()
			.map(|&suffix_dir| SPath::new(devai_dir.join(suffix_dir)).map_err(|err| err.into()))
			.collect::<Result<_>>()?;

		Ok(dirs)
	}

	pub fn get_command_agent_dirs(devai_dir: impl AsRef<Path>) -> Result<Vec<SPath>> {
		let devai_dir = devai_dir.as_ref();
		let dirs = DEVAI_COMMAND_AGENT_DIRS
			.iter()
			.map(|&suffix_dir| SPath::new(devai_dir.join(suffix_dir)).map_err(|err| err.into()))
			.collect::<Result<_>>()?;

		Ok(dirs)
	}

	pub fn get_command_agent_default_dir(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let dir = devai_dir.as_ref().join(DEVAI_AGENT_DEFAULT_DIR);
		let dir = SPath::new(dir)?;
		Ok(dir)
	}

	pub fn get_command_agent_custom_dir(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let dir = devai_dir.as_ref().join(DEVAI_AGENT_CUSTOM_DIR);
		let dir = SPath::new(dir)?;
		Ok(dir)
	}

	pub fn get_doc_dir(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let dir = devai_dir.as_ref().join(DEVAI_DOC_DIR);
		let dir = SPath::new(dir)?;
		Ok(dir)
	}

	pub fn get_doc_rhai_path(devai_dir: impl AsRef<Path>) -> Result<SPath> {
		let path = devai_dir.as_ref().join(DEVAI_DOC_RHAI_PATH);
		let path = SPath::new(path)?;
		Ok(path)
	}
}

// region:    --- Support

/// Return an option of spath tuple as (devai_parent_dir, devai_dir)
fn find_devai_parent_and_dir(from_dir: &SPath) -> Result<Option<(SPath, SPath)>> {
	let mut tmp_dir: Option<PathBuf> = Some(from_dir.into());

	while let Some(parent_dir) = tmp_dir {
		let devai_dir = parent_dir.join(".devai/");

		if devai_dir.exists() {
			return Ok(Some((SPath::new(parent_dir)?, SPath::new(devai_dir)?)));
		}

		tmp_dir = parent_dir.parent().map(|p| p.into());
	}

	Ok(None)
}

// endregion: --- Support
