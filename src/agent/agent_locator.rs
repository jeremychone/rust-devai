//! Locate agent from
//!

use crate::agent::agent_ref::{AgentRef, PackRef, PartialAgentRef};
use crate::agent::{Agent, AgentDoc, AgentOptions};
use crate::dir_context::{DirContext, PathResolver, find_pack_dirs};
use crate::support::tomls::parse_toml;
use crate::{Error, Result};
use simple_fs::{SPath, read_to_string};

pub fn find_agent(name: &str, dir_context: &DirContext) -> Result<Agent> {
	let partial_agent_ref = PartialAgentRef::new(name);

	let base_options = load_and_merge_configs_agent_options(dir_context)?;

	// -- For now, if end with .aip, we try to find direct
	let agent = match partial_agent_ref {
		PartialAgentRef::LocalPath(local_path) => {
			let path = dir_context.resolve_path(&local_path, PathResolver::CurrentDir)?;
			let possible_paths = possible_aip_paths(path.clone(), false);
			let found_path = possible_paths.into_iter().find(|p| p.exists()).ok_or_else(|| {
				Error::custom(format!(
					"No agent found for local path: '{}'\n   (full path: {})",
					local_path,
					path.to_str()
				))
			})?;
			let doc = AgentDoc::from_file(found_path)?;

			let agent_ref = AgentRef::LocalPath(local_path.to_string());

			doc.into_agent(name, agent_ref, base_options)?
		}
		PartialAgentRef::PackRef(pack_ref) => {
			let pack_dirs = find_pack_dirs(dir_context, pack_ref.namespace.as_deref(), Some(&pack_ref.name))?;

			// -- in case > 1, for now, no support
			if pack_dirs.len() > 1 {
				return Err(Error::custom(format!(
					"Multiple aipack matches for {pack_ref}. Name choose directly:\n{}\n",
					pack_dirs.iter().map(|p| p.to_string()).collect::<Vec<String>>().join("\n")
				)));
			}

			// -- Get the pack dir
			let Some(pack_dir) = pack_dirs.into_iter().next() else {
				return Err(Error::custom(format!("No aipack matches for {pack_ref}.")));
			};

			// -- Find the aip path
			// Note: if it is None, the pack_dir, then, we have the as_dir to avoid do the dir.aip
			let (aip_path, as_dir) = match pack_ref.sub_path.as_deref() {
				Some(sub_path) => (pack_dir.path.join_str(sub_path), false),
				None => (pack_dir.path.clone(), true),
			};

			let possible_aip_paths = possible_aip_paths(aip_path, as_dir);
			let Some(found_path) = possible_aip_paths.into_iter().find(|p| p.exists()) else {
				return Err(Error::custom(format!("No agent files  matches for {pack_ref}")));
			};

			// -- Buid the final agent_ref with the resolved namespace
			// TODO: Need to cleanup this strategy. Perhaps have PartialPackRef, and PackRef with namespace and pack_name
			let agent_ref = AgentRef::PackRef(PackRef::from_partial(pack_dir, pack_ref));

			// -- Build and return the agent
			let doc = AgentDoc::from_file(found_path)?;
			doc.into_agent(name, agent_ref, base_options)?
		}
	};

	Ok(agent)
}

// region:    --- Support

/// Returns the ossible .aip path for a given path
///
/// - `as_dir` allows to treat the path as dir even if it does not end with /
///
/// NOTE: This does not test if the files or path exists
///       Just give the possible path, which then need to be tested
pub fn possible_aip_paths(path: SPath, as_dir: bool) -> Vec<SPath> {
	let path_str = path.to_str();
	// if end with .aip, then, direct path, so, this is it
	if path_str.ends_with(".aip") {
		return vec![path];
	}

	// if a dir, then, just add `main.aip` per convention
	if as_dir || path_str.ends_with('/') {
		vec![path.join_str("main.aip")]
	}
	// otherwise, we have to possible paths add .aip and another with /main.aip
	else {
		vec![SPath::from(format!("{path_str}.aip")), path.join_str("main.aip")]
	}
}

/// Loads the base agent configuration.
///
/// FIXME: ->> Will need to merge the .aipack-base/config.toml with the .aipack/config.toml
pub fn load_and_merge_configs_agent_options(dir_context: &DirContext) -> Result<AgentOptions> {
	let config_paths = dir_context.aipack_paths().get_wks_config_toml_paths()?;

	let mut all_options = Vec::new();

	for config_path in config_paths {
		let config_content = read_to_string(&config_path)?;
		let config_value = parse_toml(&config_content)?;

		let options = AgentOptions::from_config_value(config_value).map_err(|err| Error::Config {
			path: config_path.to_string(),
			reason: err.to_string(),
		})?;
		all_options.push(options);
	}
	let mut options: Option<AgentOptions> = None;
	for item_options in all_options {
		options = match options {
			Some(options) => Some(options.merge(item_options)?),
			None => Some(item_options),
		}
	}

	let Some(options) = options else {
		return Err(Error::custom("No agent options found"));
	};

	Ok(options)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::_test_support::assert_contains;
	use crate::run::Runtime;
	use simple_fs::SPath;

	// region:    --- find_agent

	#[test]
	fn test_agent_locator_find_agent_ns_with_ns_pack_repo_wks() -> Result<()> {
		// -- Setup & Fixtures
		// agent name, file_path contains
		let data = &[
			(
				"ns_a@pack_a_1",
				"/sandbox-01/.aipack/pack/custom/ns_a/pack_a_1/main.aip",
			),
			(
				"ns_a@pack_a_1/sub/agent",
				"/sandbox-01/.aipack/pack/custom/ns_a/pack_a_1/sub/agent.aip",
			),
			(
				"ns_a@pack_a_1/sub/agent.aip",
				"/sandbox-01/.aipack/pack/custom/ns_a/pack_a_1/sub/agent.aip",
			),
			(
				"ns_a@pack_a_1/sub",
				"/sandbox-01/.aipack/pack/custom/ns_a/pack_a_1/sub/main.aip",
			),
			(
				"ns_a@pack_a_2/another-agent",
				"/sandbox-01/.aipack/pack/custom/ns_a/pack_a_2/another-agent.aip",
			),
			(
				"ns_b@pack_b_1",
				"/sandbox-01/.aipack/pack/custom/ns_b/pack_b_1/main.aip",
			),
		];
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();

		// -- Check & Exec
		for (name, fx_file_path) in data {
			let agent = find_agent(name, dir_context)?;

			// -- Check
			assert_eq!(agent.name(), *name);
			assert_contains(agent.file_path(), fx_file_path);
		}

		Ok(())
	}

	#[test]
	fn test_agent_locator_find_agent_ns_with_ns_pack_repo_base_custom() -> Result<()> {
		// -- Setup & Fixtures
		// agent name, file_path contains
		let data = &[
			("ns_b@pack_b_2", ".aipack-base/pack/custom/ns_b/pack_b_2/main.aip"),
			("ns_d@pack_d_1", ".aipack-base/pack/installed/ns_d/pack_d_1/main.aip"),
		];
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();

		// -- Check & Exec
		for (name, fx_file_path) in data {
			let agent = find_agent(name, dir_context)?;

			// -- Check
			assert_eq!(agent.name(), *name);
			assert_contains(agent.file_path(), fx_file_path);
		}

		Ok(())
	}

	#[test]
	fn test_agent_locator_find_agent_local_path() -> Result<()> {
		// -- Setup & Fixtures
		// agent name, file_path contains
		let data = &[
			//
			("sub-dir-a/agent-hello-2.aip", "agent-hello-2.aip"),
			("sub-dir-a/agent-hello-2", "agent-hello-2.aip"),
			("sub-dir-a/sub-sub-dir", "main.aip"),
		];
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let dir_context = runtime.dir_context();

		// -- Check & Exec
		for (name, fx_file_path) in data {
			let agent = find_agent(name, dir_context)?;

			// -- Check
			assert_eq!(agent.name(), *name);
			assert_contains(agent.file_path(), fx_file_path);
		}

		Ok(())
	}

	// endregion: --- find_agent

	// region:    --- possiple_aip_paths

	#[test]
	fn test_agent_locator_possible_aip_paths_direct_aip() -> Result<()> {
		// -- Setup & Fixtures
		let path_str = "agent.aip";
		let path = SPath::from(path_str);

		// -- Exec
		let paths = possible_aip_paths(path, false);

		// -- Check
		// When the input ends with ".aip", it should return the direct path.
		assert_eq!(paths.len(), 1);
		assert_eq!(paths[0].to_str(), path_str);

		Ok(())
	}

	#[test]
	fn test_agent_locator_possible_aip_paths_dir() -> Result<()> {
		// -- Setup & Fixtures
		let path_str = "directory/";
		let path = SPath::from(path_str);

		// -- Exec
		let paths = possible_aip_paths(path, false);

		// -- Check
		// When the input is a directory (ends with '/'), it should return a single path with "main.aip" appended.
		assert_eq!(paths.len(), 1);
		assert_eq!(paths[0].to_str(), "directory/main.aip");

		Ok(())
	}

	#[test]
	fn test_agent_locator_possible_aip_paths_regular() -> Result<()> {
		// -- Setup & Fixtures
		let path_str = "regular_path";
		let path = SPath::from(path_str);

		// -- Exec
		let paths = possible_aip_paths(path, false);

		// -- Check
		// Should return two possibilities:
		// 1. Append .aip to the path: "regular_path.aip"
		// 2. Append "/main.aip": "regular_path/main.aip"
		assert_eq!(paths.len(), 2);
		assert_eq!(paths[0].to_str(), "regular_path.aip");
		assert_eq!(paths[1].to_str(), "regular_path/main.aip");

		Ok(())
	}

	// endregion: --- possiple_aip_paths
}

// endregion: --- Tests
