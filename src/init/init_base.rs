use crate::Result;
use crate::dir_context::aipack_base_dir;
use crate::hub::get_hub;
use crate::init::assets;
use crate::support::AsStrsExt;
use simple_fs::{SPath, ensure_dir};
use std::fs::write;
use std::io::BufRead;

/// `force` - except for `config.toml`
pub async fn init_base(force: bool) -> Result<()> {
	let hub = get_hub();
	// -- Check that the home dir exists

	let mut new = false;
	// -- Create the missing folders

	let base_dir = aipack_base_dir()?;
	if ensure_dir(&base_dir)? {
		new = true;
	}

	// tokio sleep 1ms
	// UGLY - Somehow if we do not sleep here, it does not print message when aipack-base get created from scratch
	tokio::time::sleep(std::time::Duration::from_millis(1)).await;

	if new {
		hub.publish(format!("\n=== {} '{}'", "Initializing ~/.aipack-base at", base_dir))
			.await;
	} else {
		hub.publish(format!("\n=== {} '{}'", "Updating ~/.aipack-base at", base_dir))
			.await;
	}

	// -- Determine version
	let is_new_version = check_is_new_version(&base_dir).await?;
	// if new version, then, force update
	let force = is_new_version || force;

	// -- Create the config file (only if not present)
	// Note: For now, "config.toml" is not force, no matter what
	let config_path = base_dir.join_str("config.toml");
	if !config_path.exists() {
		let config_zfile = assets::extract_base_config_toml_zfile()?;
		write(&config_path, config_zfile.content)?;
		hub.publish(format!("-> {:<18} '{}'", "Create config file", config_path)).await;
	}

	// -- Init the doc
	let doc_paths = assets::extract_base_doc_file_paths()?;
	assets::update_files("base", &base_dir, &doc_paths.x_as_strs(), force).await?;

	// -- Init the pack path
	let pack_paths = assets::extract_base_pack_file_paths()?;
	assets::update_files("base", &base_dir, &pack_paths.x_as_strs(), force).await?;

	// -- Display message
	hub.publish("=== DONE\n").await;
	// yield
	tokio::task::yield_now().await;

	Ok(())
}

/// Check is the `.aipack/version.txt` is present,
/// - read the first line, and compare with current version
/// - if match current version all good.
/// - if not recreate file with version,
async fn check_is_new_version(base_dir: &SPath) -> Result<bool> {
	let version_path = base_dir.join_str("version.txt");

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

This file is used to keep track of the version and compare it during each `aip ...` execution.
If there is no match with the current version, this file will be recreated, and the documentation and other files will be updated.
		"#,
			crate::VERSION
		);
		write(&version_path, content)?;
		get_hub().publish(format!("-> {:<18} '{}'", "Create file", version_path)).await;
	}

	Ok(is_new)
}
