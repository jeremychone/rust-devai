use crate::init::migrate_devai::migrate_devai_0_1_0_if_needed;
use crate::Result;
use simple_fs::{ensure_dir, list_files};
use std::collections::HashSet;
use std::fs::write;
use std::path::Path;

const DEVAI_DIR: &str = ".devai";

// -- Agents
pub const DEVAI_AGENT_DEFAULTS_DIR: &str = ".devai/default/command-agent";
pub const DEVAI_AGENT_CUSTOMS_DIR: &str = ".devai/custom/command-agent";
const AGENT_MD_PROOF_RUST_COMMENTS_CONTENT: &str = include_str!("../../_base/agents/proof-rust-comments.devai");

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
	ensure_dir(DEVAI_AGENT_DEFAULTS_DIR)?;
	ensure_dir(DEVAI_AGENT_CUSTOMS_DIR)?;

	// -- migrate_devai_0_1_0_if_needed
	migrate_devai_0_1_0_if_needed()?;

	// -- create the default command agents if not present
	let existing_files = list_files(DEVAI_AGENT_DEFAULTS_DIR, Some(&["*.devai"]), None)?;
	let existing_names: HashSet<&str> = existing_files.iter().map(|f| f.file_name()).collect();

	for e_file in get_embedded_agent_files() {
		if !existing_names.contains(e_file.name) {
			let path = Path::new(DEVAI_AGENT_DEFAULTS_DIR).join(e_file.name);
			write(&path, e_file.content)?;
		}
	}

	// -- Create the config file
	let config_path = Path::new(DEVAI_CONFIG_FILE_PATH);
	if !config_path.exists() {
		write(config_path, DEVAI_CONFIG_FILE_CONTENT)?;
	}

	// -- Create the doc
	ensure_dir(DEVAI_DOC_DIR)?;
	let rhai_doc_path = Path::new(DEVAI_DOC_RHAI_PATH);
	if !rhai_doc_path.exists() {
		write(rhai_doc_path, DEVAI_DOC_RHAI_CONTENT)?;
	}

	Ok(())
}

// region:    --- EmbeddedAgentFile

pub(super) struct EmbeddedAgentFile {
	pub name: &'static str,
	pub content: &'static str,
}

pub(super) fn get_embedded_agent_files() -> &'static [&'static EmbeddedAgentFile] {
	&[&EmbeddedAgentFile {
		name: "proof-rust-comments.devai",
		content: AGENT_MD_PROOF_RUST_COMMENTS_CONTENT,
	}]
}

// endregion: --- EmbeddedAgentFile
