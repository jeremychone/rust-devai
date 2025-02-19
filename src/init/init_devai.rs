use crate::hub::get_hub;
use crate::init::assets::{
	extract_workspace_config_toml_zfile, extract_workspace_default_file_paths, extract_workspace_doc_file_paths,
	extract_workspace_zfile,
};

use crate::init::migrate_devai::migrate_devai_0_5_0_if_needed;
use crate::run::{find_workspace_dir, DevaiDir, DirContext};
use crate::support::files::current_dir;
use crate::support::AsStrsExt;
use crate::Result;
use simple_fs::{ensure_dir, list_files, SPath};
use std::collections::HashSet;
use std::fs::write;
use std::io::BufRead;

// -- Doc Content
// const DEVAI_DOC_LUA_CONTENT: &str = include_str!("../../_init/doc/lua.md");

/// Note: The `show_info_always` will ensure that even if the `.devai/` is found, it will print the message
///       This is useful for the `devai init` to always show the status, but not on `devai run`
pub async fn init_devai_files(ref_dir: Option<&str>, show_info_always: bool) -> Result<DirContext> {
	let hub = get_hub();

	let workspace_dir = if let Some(dir) = ref_dir {
		SPath::new(dir)?
	} else if let Some(path) = find_workspace_dir(current_dir()?)? {
		path
	} else {
		current_dir()?
	};

	let workspace_dir = workspace_dir.canonicalize()?;

	let devai_dir = DevaiDir::from_parent_dir(&workspace_dir)?;

	// -- Display the heading
	if devai_dir.exists() {
		if show_info_always {
			hub.publish("==== Initializing .devai/").await;
			hub.publish(format!(
				"-- Parent path: '{}'\n   (`.devai/` already exists. Will create missing files)",
				workspace_dir
			))
			.await;
		}
	} else {
		hub.publish("==== Initializing .devai/").await;
		hub.publish(format!(
			"-- Parent path: '{}'\n   (`.devai/` will be created at this path)",
			workspace_dir
		))
		.await;
	}

	ensure_dir(&devai_dir)?;
	let is_new_version = check_is_new_version(&devai_dir).await?;
	create_or_refresh_devai_files(&devai_dir, is_new_version).await?;

	let dir_context = DirContext::new(devai_dir)?;

	if show_info_always {
		hub.publish("-- DONE").await;
	}

	Ok(dir_context)
}

/// Check is the `.devai/version.txt` is present,
/// - read the first line, and compare with current version
/// - if match current version all good.
/// - if not recreate file with version,
async fn check_is_new_version(devai_dir: &DevaiDir) -> Result<bool> {
	let version_path = devai_dir.devai_dir_full_path().join("version.txt")?;

	let mut is_new = true;

	// -- If exists, determine if is_new
	if version_path.exists() {
		// read efficiently only the first line of  version_path
		let mut reader = simple_fs::get_buf_reader(&version_path)?;
		let mut first_line = String::new();
		if reader.read_line(&mut first_line)? > 0 {
			let version_in_file = first_line.trim();
			is_new = version_in_file != crate::VERSION;
		}
	}

	// -- If is_new, rereate the file
	if is_new {
		let content = format!(
			r#"{}

DO NOT EDIT.

This file is used to keep track of the version and compare it during each `devai ...` execution.
If there is no match with the current version, this file will be recreated, and the documentation and other files will be updated.
		"#,
			crate::VERSION
		);
		write(&version_path, content)?;
		get_hub()
			.publish(format!(
				"-> {:<18} '{}'",
				"Create file",
				version_path.diff(devai_dir.workspace_dir())?
			))
			.await;
	}

	Ok(is_new)
}

/// Create or refresh missing files in a devai directory
async fn create_or_refresh_devai_files(devai_dir: &DevaiDir, is_new_version: bool) -> Result<()> {
	let hub = get_hub();

	let workspace_dir = devai_dir.workspace_dir();

	ensure_dir(devai_dir)?;

	// -- Create the config file
	let config_path = devai_dir.get_config_toml_path()?;

	if !config_path.exists() {
		let config_zfile = extract_workspace_config_toml_zfile()?;
		write(&config_path, config_zfile.content)?;
		hub.publish(format!(
			"-> {:<18} '{}'",
			"Create config file",
			config_path.diff(workspace_dir)?
		))
		.await;
	}

	// -- migrate_devai_0_1_0_if_needed
	migrate_devai_0_5_0_if_needed(devai_dir).await?;

	// -- Create the default agent dir
	let devai_agent_default_dir = devai_dir.get_default_agent_dir()?;
	ensure_dir(devai_agent_default_dir)?;
	ensure_dir(devai_dir.get_custom_agent_dir()?)?;
	ensure_dir(devai_dir.get_default_new_template_dir()?)?;

	// -- Create the lua dir
	let devai_custom_lua_dir = devai_dir.get_lua_custom_dir()?;
	ensure_dir(devai_custom_lua_dir)?;

	// -- Create the default
	let wks_default_paths = extract_workspace_default_file_paths()?;
	update_wks_files(
		workspace_dir,
		devai_dir.devai_dir_full_path(),
		&wks_default_paths.x_as_strs(),
		false, // never force update for now
	)
	.await?;

	// -- Create the documentation
	let wks_dock_paths = extract_workspace_doc_file_paths()?;
	update_wks_files(
		workspace_dir,
		devai_dir.devai_dir_full_path(),
		&wks_dock_paths.x_as_strs(),
		is_new_version, // force update if is_new_version
	)
	.await?;

	Ok(())
}

// region:    --- Support

async fn update_wks_files(
	wks_dir: &SPath,
	devai_dir: &SPath,
	wks_file_paths: &[&str],
	force_update: bool,
) -> Result<()> {
	let existing_files = list_files(devai_dir, Some(&["**/*.devai", "**/*.lua", "**/*.md"]), None)?;

	let existing_names: HashSet<String> = existing_files
		.iter()
		.filter_map(|f| f.diff(devai_dir).ok().map(|p| p.to_string()))
		.collect();

	for &wks_file_path in wks_file_paths {
		if force_update || !existing_names.contains(wks_file_path) {
			let dest_rel_path = SPath::from(wks_file_path);
			let dest_path = SPath::new(devai_dir)?.join_str(dest_rel_path.to_str());
			// if the rel_path had a parent
			if let Some(parent_dir) = dest_rel_path.parent() {
				let parent_dir = devai_dir.join(parent_dir)?;
				ensure_dir(parent_dir)?;
			}
			let zfile = extract_workspace_zfile(dest_rel_path.to_str())?;
			write(&dest_path, zfile.content)?;
			get_hub()
				.publish(format!("-> {:<18} '{}'", "Create file", dest_path.diff(wks_dir)?))
				.await;
		}
	}

	Ok(())
}

// endregion: --- Support
