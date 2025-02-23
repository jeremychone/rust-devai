use crate::Result;
use crate::run::{DirContext, RepoKind};
use crate::support::files::list_dirs;
use crate::support::paths;
use simple_fs::SPath;

#[derive(Debug)]
pub struct PackDir {
	pub repo_kind: RepoKind,
	pub namespace: String,
	pub pack_name: String,
	pub abs_path: SPath,
}

impl PackDir {
	pub fn new(repo_kind: RepoKind, namespace: impl Into<String>, abs_path: SPath) -> Self {
		let namespace = namespace.into();
		let pack_name = abs_path.name().to_string();
		Self {
			repo_kind,
			namespace,
			abs_path,
			pack_name,
		}
	}
}

impl PackDir {
	pub fn pretty_path(&self) -> String {
		let last_five = paths::path_last_components(&self.abs_path, 5);
		let prefix = match self.repo_kind {
			RepoKind::WksCustom => "",
			RepoKind::BaseCustom => "~/",
			RepoKind::BaseInstalled => "~/",
		};
		format!("{}{}", prefix, last_five)
	}
}
impl std::fmt::Display for PackDir {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.namespace, self.pack_name)
	}
}

/// Get the matching pack_dir for this namespace and pack_name
/// - if no namespace, then, will return all of the matching PackDir with this pack_name
/// - if a namespace, will return only the first matching one (following the custom/installed preferences)
pub fn find_pack_dirs(dir_context: &DirContext, ns: Option<&str>, pack_name: Option<&str>) -> Result<Vec<PackDir>> {
	let aipack_paths = dir_context.aipack_paths();

	let repo_dirs = aipack_paths.get_pack_repo_dirs()?;

	let mut pack_dirs = Vec::new();

	for repo_dir in repo_dirs {
		let repo_kind = repo_dir.kind;

		match (ns, pack_name) {
			(Some(ns_name), Some(pack_name)) => {
				let ns_dirs = list_dirs(repo_dir.path(), 1, true);
				let found_ns_dir = ns_dirs.into_iter().find(|ns_dir| ns_dir.name() == ns_name);

				if let Some(ns_dir) = found_ns_dir {
					let aipack_dirs = list_dirs(&ns_dir, 1, true);
					let found_pack_dir = aipack_dirs.into_iter().find(|aipack_dir| aipack_dir.name() == pack_name);
					if let Some(aipack_dir) = found_pack_dir {
						// NOTE: Since direct match, just return this one
						pack_dirs.push(PackDir::new(repo_kind, ns_name, aipack_dir));
						break;
					}
				}
			}
			(ns, pack_name) => {
				let pack_dir_paths = list_dirs(repo_dir.path(), 2, true);
				for pack_path in pack_dir_paths {
					let path_pack_name = pack_path.name();
					let path_ns = pack_path.parent_name();

					// compute if we should include or not (if input is none, then, match, hence the unwrap_or true)
					let pass = match (ns, pack_name) {
						(None, None) => true,
						(Some(ns), None) => ns == path_ns,
						(None, Some(pack_name)) => pack_name == path_pack_name,
						(Some(ns), Some(pack_name)) => ns == path_ns && pack_name == path_pack_name,
					};

					if pass {
						let path_ns = path_ns.to_string();
						pack_dirs.push(PackDir::new(repo_kind, path_ns, pack_path));
					}
				}
			}
		}
	}

	Ok(pack_dirs)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::assert_contains;
	use crate::run::Runtime;
	use crate::support::AsStrsExt;
	use crate::support::paths::path_last_components;

	/// Note - In this tests, we do not use the PackDir::pretty() to not have to change those test if pretty changes.

	#[test]
	fn test_pack_dir_find_pack_dirs_all() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let expected = vec![
			"ns_a@pack_a_1 - .aipack/pack/custom/ns_a/pack_a_1",
			"ns_a@pack_a_2 - .aipack/pack/custom/ns_a/pack_a_2",
			"ns_b@pack_b_1 - .aipack/pack/custom/ns_b/pack_b_1",
			"ns_b@pack_b_1 - .aipack-base/pack/custom/ns_b/pack_b_1",
			"ns_b@pack_b_2 - .aipack-base/pack/custom/ns_b/pack_b_2",
			"ns_b@pack_b_2 - .aipack-base/pack/installed/ns_b/pack_b_2",
			"ns_d@pack_d_1 - .aipack-base/pack/installed/ns_d/pack_d_1",
		];

		// -- Exec
		let pack_dirs = find_pack_dirs(runtime.dir_context(), None, None)?;

		// -- Check
		let pack_dir_refs = pack_dir_into_strs(pack_dirs);
		let pack_dir_refs = pack_dir_refs.x_as_strs();
		for pack_dir_ref in pack_dir_refs {
			assert_contains(&expected, pack_dir_ref);
		}

		Ok(())
	}

	#[test]
	fn test_pack_dir_find_pack_dirs_ns() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let expected = vec![
			"ns_b@pack_b_1 - .aipack/pack/custom/ns_b/pack_b_1",
			"ns_b@pack_b_1 - .aipack-base/pack/custom/ns_b/pack_b_1",
			"ns_b@pack_b_2 - .aipack-base/pack/custom/ns_b/pack_b_2",
			"ns_b@pack_b_2 - .aipack-base/pack/installed/ns_b/pack_b_2",
		];

		// -- Exec
		let pack_dirs = find_pack_dirs(runtime.dir_context(), Some("ns_b"), None)?;

		// -- Check
		let pack_dir_refs = pack_dir_into_strs(pack_dirs);
		let pack_dir_refs = pack_dir_refs.x_as_strs();
		for pack_dir_ref in pack_dir_refs {
			assert_contains(&expected, pack_dir_ref);
		}

		Ok(())
	}

	#[test]
	fn test_pack_dir_find_pack_dirs_pack_name() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01()?;
		let expected = vec![
			"ns_b@pack_b_1 - .aipack/pack/custom/ns_b/pack_b_1",
			"ns_b@pack_b_1 - .aipack-base/pack/custom/ns_b/pack_b_1",
		];

		// -- Exec
		let pack_dirs = find_pack_dirs(runtime.dir_context(), None, Some("pack_b_1"))?;

		// -- Check
		let pack_dir_refs = pack_dir_into_strs(pack_dirs);
		let pack_dir_refs = pack_dir_refs.x_as_strs();
		for pack_dir_ref in pack_dir_refs {
			assert_contains(&expected, pack_dir_ref);
		}

		Ok(())
	}

	// region:    --- Support

	fn pack_dir_into_strs(pack_dirs: Vec<PackDir>) -> Vec<String> {
		pack_dirs
			.into_iter()
			.map(|p| {
				format!(
					"{}@{} - {}",
					p.namespace,
					p.pack_name,
					path_last_components(&p.abs_path, 5)
				)
			})
			.collect::<Vec<_>>()
	}
	// endregion: --- Support
}

// endregion: --- Tests
