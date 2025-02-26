use crate::Result;
use crate::dir_context::{AipackPaths, DirContext, find_wks_dir};
use crate::hub::get_hub;
use crate::init::assets::{self, extract_workspace_config_toml_zfile};
use crate::support::AsStrsExt as _;
use crate::support::files::current_dir;
use simple_fs::{SPath, ensure_dir};
use std::fs::write;

// -- Doc Content
/// Note: The `show_info_always` will ensure that even if the `.aipack/` is found, it will print the message
///       This is useful for the `aip init` to always show the status
pub async fn init_wks(ref_dir: Option<&str>, show_info_always: bool) -> Result<DirContext> {
	let hub = get_hub();

	let wks_dir = if let Some(dir) = ref_dir {
		SPath::new(dir)?
	} else if let Some(path) = find_wks_dir(current_dir()?)? {
		path
	} else {
		current_dir()?
	};

	let wks_dir = wks_dir.canonicalize()?;

	let aipack_dir = AipackPaths::from_wks_dir(&wks_dir)?;

	// -- Display the heading
	if aipack_dir.wks_aipack_dir().exists() {
		if show_info_always {
			hub.publish("\n=== Initializing .aipack/").await;
			hub.publish(format!(
				"-- Parent path: '{}'\n   (`.aipack/` already exists. Will create missing files)",
				wks_dir
			))
			.await;
		}
	} else {
		hub.publish("\n=== Initializing .aipack/").await;
		hub.publish(format!(
			"-- Parent path: '{}'\n   (`.aipack/` will be created at this path)",
			wks_dir
		))
		.await;
	}

	// -- Init or refresh
	create_or_refresh_wks_files(&aipack_dir).await?;

	if show_info_always {
		hub.publish("=== DONE\n").await;
	}

	// -- Return
	let dir_context = DirContext::new(aipack_dir)?;

	Ok(dir_context)
}

/// Create or refresh missing files in a aipack directory
/// - create `.aipack/config.toml` if not present.
/// - ensure `.aipack/pack/custom/` to show use how to create per workspace agent pack
async fn create_or_refresh_wks_files(aipack_dir: &AipackPaths) -> Result<()> {
	let hub = get_hub();

	let wks_dir = aipack_dir.wks_dir();
	let wks_aipack_dir = aipack_dir.wks_aipack_dir();

	ensure_dir(wks_aipack_dir)?;

	// -- Create the config file
	let config_path = aipack_dir.get_wks_config_toml_path()?;

	if !config_path.exists() {
		let config_zfile = extract_workspace_config_toml_zfile()?;
		write(&config_path, config_zfile.content)?;
		hub.publish(format!(
			"-> {:<18} '{}'",
			"Create config file",
			config_path.diff(wks_dir)?
		))
		.await;
	}

	// -- Ensture pack/custom
	let wks_pack_custom_dir = aipack_dir.get_wks_pack_custom_dir()?;
	ensure_dir(wks_pack_custom_dir)?;

	// -- Init the workspace pack path (the custom/...)
	let pack_paths = assets::extract_workspace_pack_file_paths()?;
	let wks_aipack_dir = aipack_dir.wks_aipack_dir();
	assets::update_files("workspace", wks_aipack_dir, &pack_paths.x_as_strs(), false).await?;

	Ok(())
}
