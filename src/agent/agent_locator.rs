use crate::agent::agent_config::AgentConfig;
use crate::agent::agents::get_embedded_agent_files;
use crate::agent::{Agent, AgentDoc};
use crate::support::tomls::parse_toml;
use crate::Result;
use simple_fs::{ensure_dir, list_files, read_to_string, SFile};
use std::collections::HashSet;
use std::fs::write;
use std::path::Path;

const DEVAI_DIR: &str = ".devai";
const DEVAI_DEFAULTS_DIR: &str = ".devai/defaults";
const DEVAI_CUSTOMS_DIR: &str = ".devai/customs";

const DEVAI_CONFIG_FILE_PATH: &str = ".devai/config.toml";
const DEVAI_CONFIG_FILE_CONTENT: &str = include_str!("../../_base/config.toml");

pub fn init_agent_files() -> Result<()> {
	ensure_dir(DEVAI_DIR)?;
	ensure_dir(DEVAI_DEFAULTS_DIR)?;
	ensure_dir(DEVAI_CUSTOMS_DIR)?;

	let existing_files = list_files(DEVAI_DEFAULTS_DIR, Some(&["*.md"]), None)?;
	let existing_names: HashSet<&str> = existing_files.iter().map(|f| f.file_name()).collect();

	// -- Create the default agent files
	for e_file in get_embedded_agent_files() {
		if !existing_names.contains(e_file.name) {
			let path = Path::new(DEVAI_DEFAULTS_DIR).join(e_file.name);
			write(&path, e_file.content)?;
		}
	}

	// -- Create the config file
	let config_path = Path::new(DEVAI_CONFIG_FILE_PATH);
	if !config_path.exists() {
		write(config_path, DEVAI_CONFIG_FILE_CONTENT)?;
	}

	Ok(())
}

pub fn find_agent(name: &str) -> Result<Agent> {
	let base_config = load_base_agent()?;
	let custom_agent_doc = find_agent_doc_in_dir(name, Path::new(DEVAI_CUSTOMS_DIR))?;
	if let Some(agent_doc) = custom_agent_doc {
		return agent_doc.into_agent(base_config);
	}

	let default_agent_doc = find_agent_doc_in_dir(name, Path::new(DEVAI_DEFAULTS_DIR))?;
	if let Some(agent_doc) = default_agent_doc {
		return agent_doc.into_agent(base_config);
	}

	Err(format!("Agent '{}' not found.", name).into())
}

// region:    --- Agent Finder Support

fn find_agent_doc_in_dir(name: &str, dir: &Path) -> Result<Option<AgentDoc>> {
	let default_files = list_files(dir, Some(&["*.md"]), None)?;
	let found_file = default_files.into_iter().find(|f| match_agent(name, f));
	if let Some(found_file) = found_file {
		let doc = AgentDoc::from_file(found_file)?;

		Ok(Some(doc))
	} else {
		Ok(None)
	}
}

fn match_agent(name: &str, sfile: &SFile) -> bool {
	let file_stem = sfile.file_stem();
	let file_stem_initials = get_initials(file_stem);

	name == file_stem || name == file_stem_initials
}

fn get_initials(input: &str) -> String {
	input
		.split('-') // Split by '-'
		.map(|part| part.trim_start_matches('_')) // Remove leading underscores
		.filter_map(|part| part.chars().next()) // Get the first character of each part
		.collect() // Collect into a String
}

// endregion: --- Agent Finder Support

// region:    --- Config Loader Support

fn load_base_agent() -> Result<AgentConfig> {
	let config_path = Path::new(DEVAI_CONFIG_FILE_PATH);
	let config_content = read_to_string(config_path)?;
	let config_value = parse_toml(&config_content)?;
	let config = AgentConfig::from_value(config_value)?;
	Ok(config)
}

// endregion: --- Config Loader Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;

	#[test]
	fn test_get_initials() -> Result<()> {
		assert_eq!(get_initials("proof-read"), "pr");
		assert_eq!(get_initials("proof-comment"), "pc");
		assert_eq!(get_initials("proof"), "p");
		assert_eq!(get_initials("_proof-comment"), "pc"); // Now this returns "pc"
		assert_eq!(get_initials("proof-_comment-read"), "pcr");
		assert_eq!(get_initials("_proof-_comment-_read"), "pcr");
		assert_eq!(get_initials("a-b-c"), "abc");

		Ok(())
	}
}

// endregion: --- Tests
