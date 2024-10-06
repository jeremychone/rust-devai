use crate::agent::agent_config::AgentConfig;
use crate::agent::agents::get_embedded_agent_files;
use crate::agent::{Agent, AgentDoc};
use crate::support::tomls::parse_toml;
use crate::Result;
use simple_fs::{ensure_dir, list_files, read_to_string, SFile};
use std::collections::HashSet;
use std::fs::write;
use std::path::Path;
use strsim::levenshtein;

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
	let dirs = get_dirs();

	// Attempt to find the agent in the specified directories
	if let Some(agent_doc) = find_agent_doc_in_dir(name, &dirs)? {
		return agent_doc.into_agent(base_config);
	}

	// If not found, return an error with potential similar agents
	let similar_paths = find_similar_agent_paths(name, &dirs)?;
	let error_msg = if !similar_paths.is_empty() {
		format!(
			"Agent '{}' not found.\nDid you mean one of these?\n{}",
			name,
			similar_paths
				.iter()
				.map(agent_sfile_as_bullet)
				.collect::<Vec<String>>()
				.join("\n")
		)
	} else {
		let agent_files = list_all_agent_files()?;
		format!(
			"Agent '{}' not found.\nHere is the list of available command agents:\n{}",
			name,
			agent_files
				.iter()
				.map(agent_sfile_as_bullet)
				.collect::<Vec<String>>()
				.join("\n")
		)
	};

	Err(error_msg.into())
}

/// Lists all agent files following the precedence rules (customs first, defaults second).
/// Agent files already present in a higher priority directory are not included.
pub fn list_all_agent_files() -> Result<Vec<SFile>> {
	let dirs = get_dirs();

	let mut sfiles = Vec::new();

	let mut file_stems: HashSet<String> = HashSet::new();

	for dir in dirs {
		let files = list_files(dir, Some(&["*.md"]), None)?;
		for file in files.into_iter() {
			let stem = file.file_stem().to_string();
			if file_stems.contains(&stem) {
				continue;
			}
			sfiles.push(file);
			file_stems.insert(stem);
		}
	}

	Ok(sfiles)
}

/// Note: For now, needs to be public because of `exec_list`
pub fn agent_sfile_as_bullet(sfile: &SFile) -> String {
	let stem = sfile.file_stem();
	let initials = get_initials(stem);
	let path = sfile.to_str();
	let msg = format!("- {stem} ({initials})");
	let msg = format!("{msg:<37} - for '{path}'");

	msg
}

// region:    --- Support

fn get_dirs() -> Vec<&'static Path> {
	vec![Path::new(DEVAI_CUSTOMS_DIR), Path::new(DEVAI_DEFAULTS_DIR)]
}

/// Finds the first matching AgentDoc in the provided directories.
fn find_agent_doc_in_dir(name: &str, dirs: &[&Path]) -> Result<Option<AgentDoc>> {
	for dir in dirs {
		let files = list_files(dir, Some(&["*.md"]), None)?;
		if let Some(found_file) = files.into_iter().find(|f| match_agent(name, f)) {
			let doc = AgentDoc::from_file(found_file)?;
			return Ok(Some(doc));
		}
	}
	Ok(None)
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

/// Finds the top 3 most similar agent file paths based on Levenshtein distance.
fn find_similar_agent_paths(name: &str, dirs: &[&Path]) -> Result<Vec<SFile>> {
	let mut candidates = Vec::new();

	for dir in dirs {
		let files = list_files(dir, Some(&["*.md"]), None)?;
		for file in files {
			candidates.push(file);
		}
	}

	let mut scored_candidates: Vec<(SFile, usize)> = candidates
		.into_iter()
		.filter_map(|sfile| {
			let file_stem = sfile.file_stem();
			let distance = levenshtein(name, file_stem);
			// note might need to change this one, seems to work ok
			const MAX_DISTANCE: usize = 5;
			if distance > MAX_DISTANCE {
				None
			} else {
				Some((sfile, distance))
			}
		})
		.collect();

	// Sort by ascending distance
	scored_candidates.sort_by_key(|&(_, distance)| distance);

	// Take the top 3
	let top_three = scored_candidates.into_iter().take(3).map(|(path, _)| path).collect();

	Ok(top_three)
}

/// Loads the base agent configuration.
fn load_base_agent() -> Result<AgentConfig> {
	let config_path = Path::new(DEVAI_CONFIG_FILE_PATH);
	let config_content = read_to_string(config_path)?;
	let config_value = parse_toml(&config_content)?;
	let config = AgentConfig::from_value(config_value)?;
	Ok(config)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	type Error = Box<dyn std::error::Error>;
	type TestResult<T> = core::result::Result<T, Error>; // For tests.

	#[test]
	fn test_get_initials() -> TestResult<()> {
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
