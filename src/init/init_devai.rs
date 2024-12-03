use crate::hub::get_hub;
use crate::init::embedded_files::{
	get_embedded_command_agent_files, get_embedded_doc_files, get_embedded_new_command_agent_files,
	get_embedded_new_solo_agent_files, EmbeddedFile,
};
use crate::init::migrate_devai::migrate_devai_0_1_0_if_needed;
use crate::run::{find_workspace_dir, DevaiDir, DirContext};
use crate::support::files::current_dir;
use crate::Result;
use simple_fs::{ensure_dir, list_files, SPath};
use std::collections::HashSet;
use std::fs::write;
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

	create_or_refresh_devai_files(&devai_dir).await?;

	let dir_context = DirContext::new(devai_dir)?;

	if show_info_always {
		hub.publish("-- DONE").await;
	}

	Ok(dir_context)
}

/// Create or refresh missing files in a devai directory
async fn create_or_refresh_devai_files(devai_dir: &DevaiDir) -> Result<()> {
	let hub = get_hub();

	let workspace_dir = devai_dir.parent_dir();

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

	// -- Create the default agent files
	let devai_agent_default_dir = devai_dir.get_command_agent_default_dir()?;
	ensure_dir(devai_agent_default_dir)?;
	ensure_dir(devai_dir.get_command_agent_custom_dir()?)?;
	for dir in devai_dir.get_new_template_command_dirs()? {
		ensure_dir(dir)?;
	}
	for dir in devai_dir.get_new_template_solo_dirs()? {
		ensure_dir(dir)?;
	}

	// -- migrate_devai_0_1_0_if_needed
	migrate_devai_0_1_0_if_needed(workspace_dir, devai_dir)?;

	// -- Create the default command agents if not present
	update_devai_files(
		workspace_dir,
		devai_dir.get_command_agent_default_dir()?,
		get_embedded_command_agent_files(),
	)
	.await?;

	// -- Create the new-template command default
	update_devai_files(
		workspace_dir,
		devai_dir.get_new_template_command_default_dir()?,
		get_embedded_new_command_agent_files(),
	)
	.await?;

	// -- Create the new-template solo default
	update_devai_files(
		workspace_dir,
		devai_dir.get_new_template_solo_default_dir()?,
		get_embedded_new_solo_agent_files(),
	)
	.await?;

	// -- Create the documentation
	update_md_files(workspace_dir, devai_dir.get_doc_dir()?, get_embedded_doc_files()).await?;

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

async fn update_md_files(base_dir: &SPath, dir: impl AsRef<Path>, embedded_agent_file: &[&EmbeddedFile]) -> Result<()> {
	let dir = dir.as_ref();
	ensure_dir(dir)?;
	let existing_files = list_files(dir, Some(&["**/*.md"]), None)?;
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

// endregion: --- Support
