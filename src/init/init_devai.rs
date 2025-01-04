use crate::hub::get_hub;
use crate::init::embedded_files::{
	get_embedded_command_agent_files, get_embedded_doc_files, get_embedded_new_command_agent_files, EmbeddedFile,
};
use crate::init::migrate_devai::migrate_devai_0_5_0_if_needed;
use crate::run::{find_workspace_dir, DevaiDir, DirContext};
use crate::support::files::current_dir;
use crate::Result;
use simple_fs::{ensure_dir, list_files, SPath};
use std::collections::HashSet;
use std::fs::write;
use std::io::BufRead;
use std::path::Path;

// -- Config Content
const DEVAI_CONFIG_FILE_CONTENT: &str = include_str!("../../_init/config.toml");

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
		write(&config_path, DEVAI_CONFIG_FILE_CONTENT)?;
		hub.publish(format!(
			"-> {:<18} '{}'",
			"Create config file",
			config_path.diff(workspace_dir)?
		))
		.await;
	}

	// -- migrate_devai_0_1_0_if_needed
	migrate_devai_0_5_0_if_needed(devai_dir).await?;

	// -- Create the default agent files
	let devai_agent_default_dir = devai_dir.get_default_agent_dir()?;
	ensure_dir(devai_agent_default_dir)?;
	ensure_dir(devai_dir.get_custom_agent_dir()?)?;
	for dir in devai_dir.get_new_template_command_dirs()? {
		ensure_dir(dir)?;
	}

	// -- Create the lua
	let devai_custom_lua_dir = devai_dir.get_lua_custom_dir()?;
	ensure_dir(devai_custom_lua_dir)?;

	// -- Create the default command agents if not present
	update_devai_files(
		workspace_dir,
		devai_dir.get_default_agent_dir()?,
		get_embedded_command_agent_files(),
	)
	.await?;

	// -- Create the new-template command default
	update_devai_files(
		workspace_dir,
		devai_dir.get_default_new_template_dir()?,
		get_embedded_new_command_agent_files(),
	)
	.await?;

	// -- Create the documentation
	update_md_files(
		workspace_dir,
		devai_dir.get_doc_dir()?,
		get_embedded_doc_files(),
		is_new_version,
	)
	.await?;

	Ok(())
}

// region:    --- Support

async fn update_devai_files(
	base_dir: &SPath,
	dir: impl AsRef<Path>,
	embedded_agent_file: &[&EmbeddedFile],
) -> Result<()> {
	let dir = dir.as_ref();
	let existing_files = list_files(dir, Some(&["**/*.devai"]), None)?;
	let existing_names: HashSet<&str> = existing_files.iter().map(|f| f.name()).collect();

	for e_file in embedded_agent_file {
		if !existing_names.contains(e_file.name) {
			let path = SPath::new(dir)?.join(e_file.name)?;
			write(&path, e_file.content)?;
			get_hub()
				.publish(format!("-> {:<18} '{}'", "Create file", path.diff(base_dir)?))
				.await;
		}
	}

	Ok(())
}

async fn update_md_files(
	base_dir: &SPath,
	dir: impl AsRef<Path>,
	embedded_agent_file: &[&EmbeddedFile],
	is_new_version: bool,
) -> Result<()> {
	let dir = dir.as_ref();
	ensure_dir(dir)?;
	let existing_files = list_files(dir, Some(&["**/*.md"]), None)?;
	let existing_names: HashSet<&str> = existing_files.iter().map(|f| f.name()).collect();

	for e_file in embedded_agent_file {
		if is_new_version || !existing_names.contains(e_file.name) {
			let path = SPath::new(dir)?.join(e_file.name)?;
			write(&path, e_file.content)?;
			get_hub()
				.publish(format!("-> {:<18} '{}'", "Create file", path.diff(base_dir)?))
				.await;
		}
	}

	Ok(())
}

// endregion: --- Support
