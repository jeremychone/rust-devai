use crate::agent::agents::get_embedded_agent_files;
use crate::agent::{Agent, AgentDoc};
use crate::Result;
use simple_fs::{ensure_dir, list_files, read_to_string, SFile};
use std::collections::HashSet;
use std::fs::write;
use std::path::Path;

const DEVAI_DIR: &str = ".devai";
const DEVAI_DEFAULTS_DIR: &str = ".devai/defaults";
const DEVAI_CUSTOMS_DIR: &str = ".devai/customs";

pub fn init_agent_files() -> Result<()> {
	ensure_dir(DEVAI_DIR)?;
	ensure_dir(DEVAI_DEFAULTS_DIR)?;
	ensure_dir(DEVAI_CUSTOMS_DIR)?;

	let existing_files = list_files(DEVAI_DEFAULTS_DIR, Some(&["*.md"]), None)?;
	let existing_names: HashSet<&str> = existing_files.iter().map(|f| f.file_name()).collect();

	for &e_file in get_embedded_agent_files() {
		if !existing_names.contains(e_file.name) {
			let path = Path::new(DEVAI_DEFAULTS_DIR).join(e_file.name);
			write(&path, e_file.content)?;
		}
	}
	Ok(())
}

pub fn find_agent(name: &str) -> Result<Agent> {
	let custom_agent = find_agent_in_dir(name, Path::new(DEVAI_CUSTOMS_DIR))?;
	if let Some(agent) = custom_agent {
		return Ok(agent);
	}

	let default_agent = find_agent_in_dir(name, Path::new(DEVAI_DEFAULTS_DIR))?;
	if let Some(agent) = default_agent {
		return Ok(agent);
	}

	Err(format!("Agent '{}' not found.", name).into())
}

// region:    --- Support

fn find_agent_in_dir(name: &str, dir: &Path) -> Result<Option<Agent>> {
	let default_files = list_files(dir, Some(&["*.md"]), None)?;
	let found_file = default_files.into_iter().find(|f| match_agent(name, f));
	if let Some(found_file) = found_file {
		let doc = AgentDoc::from_file(found_file)?;
		let agent = doc.into_agent()?;
		Ok(Some(agent))
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

// endregion: --- Support

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
