use crate::init::embedded_files::{
	get_embedded_command_agent_files, get_embedded_new_command_agent_files, get_embedded_new_solo_agent_files,
	EmbeddedFile,
};
use crate::init::migrate_devai::migrate_devai_0_1_0_if_needed;
use crate::Result;
use simple_fs::{ensure_dir, list_files};
use std::collections::HashSet;
use std::fs::write;
use std::path::Path;

const DEVAI_DIR: &str = ".devai";

// -- Agents
pub const DEVAI_AGENT_DEFAULT_DIR: &str = ".devai/default/command-agent";
pub const DEVAI_AGENT_CUSTOM_DIR: &str = ".devai/custom/command-agent";

// -- New Command Templates
pub const DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR: &str = ".devai/default/new-template/command-agent";
pub const DEVAI_NEW_CUSTOM_COMMAND_AGENT_DIR: &str = ".devai/custom/new-template/command-agent";

// -- New Solo Templates
pub const DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR: &str = ".devai/default/new-template/solo-agent";
pub const DEVAI_NEW_CUSTOM_SOLO_AGENT_DIR: &str = ".devai/custom/new-template/solo-agent";

// -- Config
pub const DEVAI_CONFIG_FILE_PATH: &str = ".devai/config.toml";
const DEVAI_CONFIG_FILE_CONTENT: &str = include_str!("../../_base/config.toml");

// -- Doc
pub const DEVAI_DOC_DIR: &str = ".devai/doc";
pub const DEVAI_DOC_RHAI_PATH: &str = ".devai/doc/rhai.md";
const DEVAI_DOC_RHAI_CONTENT: &str = include_str!("../../_base/doc/rhai.md");

pub fn init_devai_files() -> Result<()> {
	ensure_dir(DEVAI_DIR)?;

	// -- Create the default agent files
	ensure_dir(DEVAI_AGENT_DEFAULT_DIR)?;
	ensure_dir(DEVAI_AGENT_CUSTOM_DIR)?;
	ensure_dir(DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR)?;
	ensure_dir(DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR)?;

	// -- migrate_devai_0_1_0_if_needed
	migrate_devai_0_1_0_if_needed()?;

	// -- Create the default command agents if not present
	update_devai_files(DEVAI_AGENT_DEFAULT_DIR, get_embedded_command_agent_files())?;

	// -- Create the config file
	let config_path = Path::new(DEVAI_CONFIG_FILE_PATH);
	if !config_path.exists() {
		write(config_path, DEVAI_CONFIG_FILE_CONTENT)?;
	}

	// -- Create the new-template command default
	update_devai_files(
		DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR,
		get_embedded_new_command_agent_files(),
	)?;

	// -- Create the new-template solo default
	update_devai_files(DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR, get_embedded_new_solo_agent_files())?;

	// -- Create the doc
	ensure_dir(DEVAI_DOC_DIR)?;
	let rhai_doc_path = Path::new(DEVAI_DOC_RHAI_PATH);
	if !rhai_doc_path.exists() {
		write(rhai_doc_path, DEVAI_DOC_RHAI_CONTENT)?;
	}

	Ok(())
}

// region:    --- Support

fn update_devai_files(dir: &str, embedded_agent_file: &[&EmbeddedFile]) -> Result<()> {
	let existing_files = list_files(dir, Some(&["*.devai"]), None)?;
	let existing_names: HashSet<&str> = existing_files.iter().map(|f| f.file_name()).collect();

	for e_file in embedded_agent_file {
		if !existing_names.contains(e_file.name) {
			let path = Path::new(dir).join(e_file.name);
			write(&path, e_file.content)?;
		}
	}

	Ok(())
}

// endregion: --- Support
