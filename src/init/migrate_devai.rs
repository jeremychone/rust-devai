use crate::hub::get_hub;
use crate::run::DevaiDir;
use crate::Result;
use simple_fs::{ensure_dir, list_files, SPath};
use std::fs;
use std::path::Path;

pub const DEVAI_0_1_0_AGENT_DEFAULTS_DIR: &str = ".devai/defaults";
pub const DEVAI_0_1_0_AGENT_CUSTOMS_DIR: &str = ".devai/customs";

pub const DEVAI_0_1_0_DEPRECATED_DIR: &str = ".devai/_deprecated_v0_1_0";

pub fn migrate_devai_0_1_0_if_needed(base_dir: &SPath, devai_dir: &DevaiDir) -> Result<bool> {
	// -- migrate the default command agents
	let agent_default_dir = devai_dir.get_command_agent_default_dir()?;
	let defaults_migrated = migrate_agent_dir(base_dir, DEVAI_0_1_0_AGENT_DEFAULTS_DIR, agent_default_dir)?;
	archive_agent_dir(base_dir, DEVAI_0_1_0_AGENT_DEFAULTS_DIR)?;

	// -- migrate the custom command agents
	let agent_custom_dir = devai_dir.get_command_agent_custom_dir()?;
	let customs_migrated = migrate_agent_dir(base_dir, DEVAI_0_1_0_AGENT_CUSTOMS_DIR, agent_custom_dir)?;
	archive_agent_dir(base_dir, DEVAI_0_1_0_AGENT_CUSTOMS_DIR)?;

	Ok(defaults_migrated || customs_migrated)
}

/// This is a v0.1.0 to v0.1.1 migration
/// For example (from .devai/customs/.. to ./devai/custom/command-agent/..)
/// - Copy the legacy `*.md` at the root of the folder to the new target folder with `*.devai`
///    -  Only the direct decending .md files
/// - Move the whole legacy folder `.devai/customs` to the `.devai/deprecated_v0_1_0/customs`
fn migrate_agent_dir(base_dir: &SPath, old_dir: impl AsRef<Path>, dest_dir: impl AsRef<Path>) -> Result<bool> {
	let hub = get_hub();

	let old_dir = old_dir.as_ref();
	if !old_dir.exists() {
		return Ok(false);
	}

	let dest_dir = SPath::from_path(dest_dir)?;

	ensure_dir(&dest_dir)?;

	let mut at_least_one = false;

	let legacy_files = list_files(old_dir, Some(&["*.md"]), None);

	for file in legacy_files? {
		let dest_file_name = format!("{}.devai", file.stem());
		let dest_file = dest_dir.join(&dest_file_name)?;

		// we skip
		if dest_file.exists() {
			continue;
		}

		std::fs::copy(file.path(), &dest_file)?;
		hub.publish_sync(format!(
			"-- 0.1.0 to 0.2.x migration - copied '{file}' to '{}'",
			dest_file.diff(base_dir)?
		));

		if !at_least_one {
			at_least_one = true;
		}
	}

	Ok(at_least_one)
}

fn archive_agent_dir(base_dir: &SPath, old_dir: impl AsRef<Path>) -> Result<bool> {
	let old_dir = old_dir.as_ref();
	if !old_dir.exists() {
		return Ok(false);
	}

	let old_dir = SPath::from_path(old_dir)?;

	let dest_base_dir = SPath::new(DEVAI_0_1_0_DEPRECATED_DIR)?;
	ensure_dir(&dest_base_dir)?;

	let dest_dir = dest_base_dir.join(old_dir.name())?;

	fs::rename(&old_dir, &dest_dir)?;
	get_hub().publish_sync(format!(
		"-- 0.1.0 to 0.2.x migration - archived dir '{}' to '{}'",
		old_dir.diff(base_dir)?,
		dest_dir.diff(base_dir)?
	));

	Ok(true)
}
