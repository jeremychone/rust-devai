use crate::agent::agent_options::AgentOptions;
use crate::agent::{Agent, AgentDoc};
use crate::run::{DirContext, PathResolver};
use crate::support::tomls::parse_toml;
use crate::{Error, Result};
use simple_fs::{list_files, read_to_string, SFile, SPath};
use std::collections::HashSet;
use std::path::Path;
use strsim::levenshtein;

/// - `from_cli` Used to decide which "reference dir" should be used.
///              If from cli, the current_dir will be used, otherwise, the workspace_dir
pub fn find_agent(agent_name: &str, dir_context: &DirContext, mode: PathResolver) -> Result<Agent> {
	let base_config = load_base_agent_config(dir_context)?;

	let devai_dir = dir_context.devai_dir();

	// -- For now, if end with .devai, we try to find direct
	let agent_sname = SPath::new(agent_name)?;
	if agent_sname.ext() == "devai" {
		let agent_file = dir_context.resolve_path(&agent_sname, mode)?;
		let agent_file =
			SFile::try_from(agent_file).map_err(|_| Error::CommandAgentNotFound(agent_sname.to_string()))?;
		let doc = AgentDoc::from_file(agent_file)?;
		return doc.into_agent(agent_sname, base_config);
	}

	// -- Otherwise, look in the command-agent dirs
	let dirs = devai_dir.get_agent_dirs()?;
	let dirs = dirs.iter().map(|dir| dir.path()).collect::<Vec<_>>();

	// Attempt to find the agent in the specified directories
	if let Some(agent_doc) = find_agent_doc_in_dir(agent_name, &dirs)? {
		return agent_doc.into_agent(agent_name, base_config);
	}

	// If not found, return an error with potential similar agents
	let similar_paths = find_similar_agent_paths(agent_name, &dirs)?;
	let error_msg = if !similar_paths.is_empty() {
		format!(
			"Agent '{}' not found.\nDid you mean one of these?\n{}",
			agent_name,
			similar_paths
				.iter()
				.map(agent_agent_rel_as_bullet)
				.collect::<Vec<String>>()
				.join("\n")
		)
	} else {
		let agent_rels = list_all_agent_rels(dir_context)?;
		format!(
			"Agent '{}' not found.\nHere is the list of available command agents:\n{}",
			agent_name,
			agent_rels
				.iter()
				.map(agent_agent_rel_as_bullet)
				.collect::<Vec<String>>()
				.join("\n")
		)
	};

	Err(error_msg.into())
}

/// Lists all agent files following the precedence rules (customs first, defaults second).
/// Agent files already present in a higher priority directory are not included.
pub fn list_all_agent_rels(dir_context: &DirContext) -> Result<Vec<AgentRel>> {
	let dirs = dir_context.devai_dir().get_agent_dirs()?;
	let mut agent_rels = Vec::new();

	let mut file_stems: HashSet<String> = HashSet::new();

	for dir in dirs {
		let files = list_files(&dir, Some(&["**/*.devai"]), None)?;
		for file in files.into_iter() {
			let Some(agent_rel) = AgentRel::new(&dir, &file) else {
				continue;
			};

			let rel_stem = agent_rel.rel_path_stem().to_string();
			if file_stems.contains(&rel_stem) {
				continue;
			}
			agent_rels.push(agent_rel);
			file_stems.insert(rel_stem);
		}
	}

	Ok(agent_rels)
}

/// Note: For now, needs to be public because of `exec_list`
pub fn agent_agent_rel_as_bullet(agent_rel: &AgentRel) -> String {
	let rel_stem = agent_rel.rel_path_stem();
	let initials = agent_rel.initials();
	let path = agent_rel.sfile.to_str();
	let msg = format!("- {rel_stem} ({initials})");
	let msg = format!("{msg:<37} - for '{path}'");

	msg
}

// region:    --- Support
/// Finds the first matching AgentDoc in the provided directories.
fn find_agent_doc_in_dir(name: &str, dirs: &[&Path]) -> Result<Option<AgentDoc>> {
	for &dir in dirs {
		let files = list_files(dir, Some(&["**/*.devai"]), None)?;

		let mut main_files: Vec<SFile> = Vec::new();

		// First try to find the full path name like `my-agent`/`ma`, `my-mod/super`/`mm/s`
		if let Some(found_file) = files.into_iter().find(|f| {
			// Note: here we capture if the f.stem is "main" so that we can quickly match it below.
			if f.stem() == "main" {
				main_files.push(f.clone())
			}
			match_agent(dir, name, f)
		}) {
			// NOTE: Because the dirs are form the DevaiDir and might not be absolute, and relative to working dir
			//       But later, need to remove from the current_dir of the DirContext, so, needs full path
			let found_file = found_file.canonicalize()?;
			let doc = AgentDoc::from_file(found_file)?;
			return Ok(Some(doc));
		}

		// if not found, then, perhaps is the just mode agent and want to have a main
		let name = format!("{name}/main");
		for sfile in main_files {
			if match_agent(dir, &name, &sfile) {
				// NOTE: See above
				let found_file = sfile.canonicalize()?;
				let doc = AgentDoc::from_file(found_file)?;
				return Ok(Some(doc));
			}
		}
	}
	Ok(None)
}

fn match_agent(base_dir: &Path, name: &str, sfile: &SFile) -> bool {
	let Some(agent_rel) = AgentRel::new(base_dir, sfile) else {
		return false;
	};
	name == agent_rel.rel_path_stem() || name == agent_rel.initials()
}

/// The structure that represent the agent and relative path
#[derive(Debug)]
pub struct AgentRel {
	sfile: SFile,
	rel_path: SPath,
}

impl AgentRel {
	fn new(base_dir: impl AsRef<Path>, sfile: &SFile) -> Option<Self> {
		let rel_path = sfile.diff(base_dir).ok()?;
		Some(AgentRel {
			rel_path,
			sfile: sfile.clone(),
		})
	}

	/// Return the rel_path (with `.devai` extension) as str
	fn to_str(&self) -> &str {
		self.rel_path.to_str()
	}

	/// Remove the `.devai` suffix
	fn rel_path_stem(&self) -> &str {
		let rel_path_str = self.to_str();
		rel_path_str.strip_suffix(".devai").unwrap_or(rel_path_str)
	}

	fn initials(&self) -> String {
		get_initials(self.to_str())
	}
}

fn get_initials(input: &str) -> String {
	input
		.split('/') // Split by '/'
		.map(|segment| {
			segment
				.split('-') // Split by '-'
				.filter_map(|part| {
					let part = part.strip_prefix("_").unwrap_or(part);
					part.chars().next()
				}) // Get the first character of each part
				.collect::<String>() // Collect into a String
		})
		.collect::<Vec<String>>() // Collect segments into a Vec
		.join("/") // Join segments with '/'
}

/// Finds the top 3 most similar agent file paths based on Levenshtein distance.
fn find_similar_agent_paths(name: &str, dirs: &[&Path]) -> Result<Vec<AgentRel>> {
	let mut candidates = Vec::new();

	for dir in dirs {
		let files = list_files(dir, Some(&["**/*.devai"]), None)?;
		for file in files {
			if let Some(agent_rel) = AgentRel::new(dir, &file) {
				candidates.push(agent_rel);
			};
		}
	}

	let mut scored_candidates: Vec<(AgentRel, usize)> = candidates
		.into_iter()
		.filter_map(|agent_rel| {
			let agent_rel_stem = agent_rel.rel_path_stem();
			let distance = levenshtein(name, agent_rel_stem);

			// note might need to change this one, seems to work ok
			const MAX_DISTANCE: usize = 5;
			if distance > MAX_DISTANCE {
				None
			} else {
				Some((agent_rel, distance))
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
pub fn load_base_agent_config(dir_context: &DirContext) -> Result<AgentOptions> {
	let config_path = dir_context.devai_dir().get_config_toml_path()?;
	let config_content = read_to_string(&config_path)?;
	let config_value = parse_toml(&config_content)?;

	let options = AgentOptions::from_config_value(config_value).map_err(|err| Error::Config {
		path: config_path.to_string(),
		reason: err.to_string(),
	})?;
	Ok(options)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::_test_support::{run_test_agent, SANDBOX_01_DIR};
	use crate::run::Runtime;
	use simple_fs::ensure_dir;
	use value_ext::JsonValueExt;

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_get_initials() -> Result<()> {
		let test_cases: &[(&str, &str)] = &[
			("proof-read", "pr"),
			("proof-comment", "pc"),
			("proof", "p"),
			("_proof-comment", "pc"),
			("proof-_comment-read", "pcr"),
			("_proof-_comment-_read", "pcr"),
			("a-b-c", "abc"),
			("hello/world", "h/w"),
			("hello/big-world", "h/bw"),
			("nice-hello/big-world", "nh/bw"),
			("nice-hello/_big-world", "nh/bw"),
		];

		for &(content, expected) in test_cases {
			assert_eq!(get_initials(content), expected);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_find_command_agent_direct_and_validate_ctx() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let fx_agent_name = "./agent-script/agent-ctx-reflect.devai";

		// -- Exec
		let agent = find_agent(fx_agent_name, runtime.dir_context(), PathResolver::CurrentDir)?;
		let res = run_test_agent(&runtime, &agent).await?;

		// -- Check
		// workspace_dir
		let workspace_dir = res.x_get_as::<&str>("WORKSPACE_DIR")?;
		assert!(Path::new(workspace_dir).is_absolute(), "workspace_dir must be absolute");
		assert!(
			workspace_dir.ends_with("tests-data/sandbox-01"),
			"WORKSPACE_DIR must end with 'tests-data/sandbox-01'"
		);

		// devai dir
		assert_eq!(res.x_get_as::<&str>("DEVAI_DIR")?, "./.devai");

		assert_eq!(res.x_get_as::<&str>("AGENT_NAME")?, fx_agent_name);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_PATH")?, fx_agent_name);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_DIR")?, "./agent-script");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_NAME")?, "agent-ctx-reflect.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_STEM")?, "agent-ctx-reflect");

		Ok(())
	}

	#[tokio::test]
	async fn test_find_command_agent_custom_and_validate_ctx() -> Result<()> {
		// -- Setup & Fixtures
		// TODO: Probably need to run the init in sandbox_01
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		ensure_dir("tests-data/sandbox-01/.devai/custom/agent")?;
		std::fs::copy(
			Path::new(SANDBOX_01_DIR).join("agent-script/agent-ctx-reflect.devai"),
			"tests-data/sandbox-01/.devai/custom/agent/command-ctx-reflect.devai",
		)?;
		// -- Exec
		let agent = find_agent("command-ctx-reflect", runtime.dir_context(), PathResolver::CurrentDir)?;
		let res = run_test_agent(&runtime, &agent).await?;

		// -- Check
		// workspace_dir
		let workspace_dir = res.x_get_as::<&str>("WORKSPACE_DIR")?;
		assert!(Path::new(workspace_dir).is_absolute(), "workspace_dir must be absolute");
		assert!(
			workspace_dir.ends_with("tests-data/sandbox-01"),
			"WORKSPACE_DIR must end with 'tests-data/sandbox-01'"
		);

		// devai dir
		assert_eq!(res.x_get_as::<&str>("DEVAI_DIR")?, "./.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_NAME")?, "command-ctx-reflect");
		assert_eq!(
			res.x_get_as::<&str>("AGENT_FILE_PATH")?,
			"./.devai/custom/agent/command-ctx-reflect.devai"
		);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_DIR")?, "./.devai/custom/agent");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_NAME")?, "command-ctx-reflect.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_STEM")?, "command-ctx-reflect");

		Ok(())
	}

	#[tokio::test]
	async fn test_find_command_agent_nested_full_name() -> Result<()> {
		// -- Setup & Fixtures
		// TODO: Probably need to run the init in sandbox_01
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		ensure_dir("tests-data/sandbox-01/.devai/custom/agent/sub-dir/")?;
		std::fs::copy(
			Path::new(SANDBOX_01_DIR).join("agent-script/agent-ctx-reflect.devai"),
			"tests-data/sandbox-01/.devai/custom/agent/sub-dir/sub-agent.devai",
		)?;
		// -- Exec
		let agent = find_agent("sub-dir/sub-agent", runtime.dir_context(), PathResolver::CurrentDir)?;
		let res = run_test_agent(&runtime, &agent).await?;

		// -- Check
		// workspace_dir
		let workspace_dir = res.x_get_as::<&str>("WORKSPACE_DIR")?;
		assert!(Path::new(workspace_dir).is_absolute(), "workspace_dir must be absolute");
		assert!(
			workspace_dir.ends_with("tests-data/sandbox-01"),
			"WORKSPACE_DIR must end with 'tests-data/sandbox-01'"
		);

		// devai dir
		assert_eq!(res.x_get_as::<&str>("DEVAI_DIR")?, "./.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_NAME")?, "sub-dir/sub-agent");
		assert_eq!(
			res.x_get_as::<&str>("AGENT_FILE_PATH")?,
			"./.devai/custom/agent/sub-dir/sub-agent.devai"
		);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_DIR")?, "./.devai/custom/agent/sub-dir");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_NAME")?, "sub-agent.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_STEM")?, "sub-agent");

		Ok(())
	}

	#[tokio::test]
	async fn test_find_command_agent_nested_main() -> Result<()> {
		// -- Setup & Fixtures
		// TODO: Probably need to run the init in sandbox_01
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		ensure_dir("tests-data/sandbox-01/.devai/custom/agent/mod-agent/")?;
		std::fs::copy(
			Path::new(SANDBOX_01_DIR).join("agent-script/agent-ctx-reflect.devai"),
			"tests-data/sandbox-01/.devai/custom/agent/mod-agent/main.devai",
		)?;
		// -- Exec
		let agent = find_agent("mod-agent", runtime.dir_context(), PathResolver::CurrentDir)?;
		let res = run_test_agent(&runtime, &agent).await?;

		// -- Check
		// workspace_dir
		let workspace_dir = res.x_get_as::<&str>("WORKSPACE_DIR")?;
		assert!(Path::new(workspace_dir).is_absolute(), "workspace_dir must be absolute");
		assert!(
			workspace_dir.ends_with("tests-data/sandbox-01"),
			"WORKSPACE_DIR must end with 'tests-data/sandbox-01'"
		);

		// devai dir
		assert_eq!(res.x_get_as::<&str>("DEVAI_DIR")?, "./.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_NAME")?, "mod-agent");
		assert_eq!(
			res.x_get_as::<&str>("AGENT_FILE_PATH")?,
			"./.devai/custom/agent/mod-agent/main.devai"
		);
		assert_eq!(
			res.x_get_as::<&str>("AGENT_FILE_DIR")?,
			"./.devai/custom/agent/mod-agent"
		);
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_NAME")?, "main.devai");
		assert_eq!(res.x_get_as::<&str>("AGENT_FILE_STEM")?, "main");

		Ok(())
	}
}

// endregion: --- Tests
