use super::support::open_vscode;
use crate::agent::get_solo_and_target_path;
use crate::cli::NewSoloArgs;
use crate::hub::get_hub;
use crate::run::{DirContext, PathResolver};
use crate::support::files::first_file_from_dirs;
use crate::Result;
use simple_fs::ensure_file_dir;
use std::path::Path;

/// exec for the New command
pub async fn exec_new_solo(new_config: impl Into<NewSoloConfig>, dir_context: DirContext) -> Result<()> {
	let hub = get_hub();

	let new_config = new_config.into();

	// TODO: support --template template_name
	let dirs = dir_context.devai_dir().get_new_template_solo_dirs()?;
	let dirs = dirs.iter().map(|dir| dir.to_str()).collect::<Vec<_>>();
	let template_file = first_file_from_dirs(&dirs, "default.devai")
		.ok()
		.flatten()
		.ok_or("solo agent template 'default.devai' not found")?;

	let solo_file_path = if new_config.path.ends_with(".devai") {
		new_config.path
	} else {
		format!("{}.devai", new_config.path)
	};
	let solo_file_path = Path::new(&solo_file_path);

	// if it does not exist, we create
	if !solo_file_path.exists() {
		ensure_file_dir(solo_file_path)?;
		std::fs::copy(template_file.path(), solo_file_path)?;

		hub.publish(format!(
			"-> New solo file created: {}",
			solo_file_path.to_string_lossy()
		))
		.await;
	}
	// If already exists, we publish a message, but we do not break
	else {
		hub.publish(format!(
			"-! Solo agent file '{}' already exists.",
			solo_file_path.to_string_lossy()
		))
		.await;
	}

	// We open no matter what
	if new_config.open {
		// open the target file if exists
		let (_, target_path) = get_solo_and_target_path(solo_file_path)?;
		let target_path = dir_context.resolve_path(target_path, PathResolver::CurrentDir)?;
		if target_path.exists() {
			open_vscode(target_path).await;
		}

		open_vscode(solo_file_path).await;
	}

	Ok(())
}

// region:    --- NewConfig

#[derive(Debug)]
pub struct NewSoloConfig {
	/// The path of the solo .devai of the target file (in this case .devai will be added)
	pub path: String,

	/// If the file(s) needs to be open (via code of vscode for now)
	pub open: bool,
}

impl From<NewSoloArgs> for NewSoloConfig {
	fn from(args: NewSoloArgs) -> Self {
		NewSoloConfig {
			path: args.path,
			open: args.open,
		}
	}
}

// endregion: --- NewConfig
