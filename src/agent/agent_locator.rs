use crate::agent::agent_config::AgentConfig;
use crate::agent::{Agent, AgentDoc};
use crate::support::tomls::parse_toml;
use crate::support::DirContext;
use crate::{Error, Result};
use simple_fs::{list_files, read_to_string, SFile, SPath};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use strsim::levenshtein;

pub fn find_agent(agent_path: &str, dir_context: &DirContext) -> Result<Agent> {
	let base_config = load_base_agent_config(dir_context)?;

	// -- First see if it is a direct path (starts with `./` or `/`)
	if agent_path.starts_with("./") || agent_path.starts_with("/") {
		let sfile = SFile::new(agent_path).map_err(|_| Error::CommandAgentNotFound(agent_path.to_string()))?;
		let doc = AgentDoc::from_file(sfile)?;
		return doc.into_agent(base_config);
	}

	// -- Otherwise, look in the command-agent dirs
	let dirs = DirContext::get_command_agent_dirs(dir_context.devai_dir())?;
	let dirs = dirs.iter().map(|dir| dir.path()).collect::<Vec<_>>();

	// Attempt to find the agent in the specified directories
	if let Some(agent_doc) = find_agent_doc_in_dir(agent_path, &dirs)? {
		return agent_doc.into_agent(base_config);
	}

	// If not found, return an error with potential similar agents
	let similar_paths = find_similar_agent_paths(agent_path, &dirs)?;
	let error_msg = if !similar_paths.is_empty() {
		format!(
			"Agent '{}' not found.\nDid you mean one of these?\n{}",
			agent_path,
			similar_paths
				.iter()
				.map(agent_sfile_as_bullet)
				.collect::<Vec<String>>()
				.join("\n")
		)
	} else {
		let agent_files = list_all_agent_files(dir_context)?;
		format!(
			"Agent '{}' not found.\nHere is the list of available command agents:\n{}",
			agent_path,
			agent_files
				.iter()
				.map(agent_sfile_as_bullet)
				.collect::<Vec<String>>()
				.join("\n")
		)
	};

	Err(error_msg.into())
}

/// Returns the (solo_path, target_path) tuple for a file path of either.
/// - If the path ends with `.devai`, then it is the solo path
///   - If the solo stem has an extension, then, the target path is the path without .devai
///   - If the solo stem does not have an extension, then, .md is added for the target path
/// - Otherwise, add `.devai` to the file name in the same path.
pub fn get_solo_and_target_path(path: impl Into<PathBuf>) -> Result<(SPath, SPath)> {
	let path = SPath::new(path)?;

	// returns (solo_path, target_path)
	// path is the solo_path
	let solo_and_target_path = if path.ext() == "devai" {
		let target_file_stem = path.file_stem();
		let target_file_name = if Path::new(target_file_stem).extension().is_some() {
			target_file_stem.to_string()
		} else {
			format!("{target_file_stem}.md")
		};

		let target_path = path.new_sibling(target_file_name)?;
		(path, target_path)
	}
	// path is the target_path
	else {
		let solo_path = path.new_sibling(format!("{}.devai", path.file_name()))?;
		(solo_path, path)
	};

	Ok(solo_and_target_path)
}

/// Lists all agent files following the precedence rules (customs first, defaults second).
/// Agent files already present in a higher priority directory are not included.
pub fn list_all_agent_files(dir_context: &DirContext) -> Result<Vec<SFile>> {
	let dirs = DirContext::get_command_agent_dirs(dir_context.devai_dir())?;

	let mut sfiles = Vec::new();

	let mut file_stems: HashSet<String> = HashSet::new();

	for dir in dirs {
		let files = list_files(dir, Some(&["*.devai"]), None)?;
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
/// Finds the first matching AgentDoc in the provided directories.
fn find_agent_doc_in_dir(name: &str, dirs: &[&Path]) -> Result<Option<AgentDoc>> {
	for dir in dirs {
		let files = list_files(dir, Some(&["*.devai"]), None)?;
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
		let files = list_files(dir, Some(&["*.devai"]), None)?;
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
pub fn load_base_agent_config(dir_context: &DirContext) -> Result<AgentConfig> {
	let config_path = DirContext::get_config_toml_path(dir_context.devai_dir())?;
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

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

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

	#[test]
	fn test_get_solo_and_target_path() -> Result<()> {
		let data = &[
			// (path, expected_solo_path, expected_target_path)
			("./some/file.md", "./some/file.md.devai", "./some/file.md"),
			("./some/file.md.devai", "./some/file.md.devai", "./some/file.md"),
			("./some/file.devai", "./some/file.devai", "./some/file.md"),
		];

		// -- Exec & Check
		for (path, expected_solo_path, expected_target_path) in data {
			let (solo_path, target_path) = get_solo_and_target_path(path)?;
			assert_eq!(solo_path.to_str(), *expected_solo_path);
			assert_eq!(target_path.to_str(), *expected_target_path);
		}

		Ok(())
	}
}

// endregion: --- Tests
