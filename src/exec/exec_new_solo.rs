use crate::cli::NewSoloArgs;
use crate::exec::support::first_file_from_dirs;
use crate::hub::get_hub;
use crate::init::{DEVAI_NEW_CUSTOM_SOLO_AGENT_DIR, DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR};
use crate::Result;
use simple_fs::ensure_file_dir;
use std::path::Path;

/// exec for the New command
pub async fn exec_new_solo(new_config: impl Into<NewSoloConfig>) -> Result<()> {
	let new_config = new_config.into();

	// TODO: support --template template_name
	let template_file = first_file_from_dirs(
		&[DEVAI_NEW_CUSTOM_SOLO_AGENT_DIR, DEVAI_NEW_DEFAULT_SOLO_AGENT_DIR],
		"default.devai", // for now, just look for default.devai
	)
	.ok()
	.flatten()
	.ok_or("solo agent template 'default.devai' not found")?;

	let solo_file_path = if new_config.path.ends_with(".devai") {
		new_config.path
	} else {
		format!("{}.devai", new_config.path)
	};
	let solo_file_path = Path::new(&solo_file_path);

	if solo_file_path.exists() {
		return Err(format!("Solo agent file '{}' already exists.", solo_file_path.to_string_lossy()).into());
	}

	ensure_file_dir(solo_file_path)?;

	std::fs::copy(template_file.path(), solo_file_path)?;

	get_hub()
		.publish(format!(
			"-> New solo file created: {}",
			solo_file_path.to_string_lossy()
		))
		.await;

	Ok(())
}

// region:    --- NewConfig

#[derive(Debug)]
pub struct NewSoloConfig {
	/// The path of the solo .devai of the target file (in this case .devai will be added)
	pub path: String,
}

impl From<NewSoloArgs> for NewSoloConfig {
	fn from(args: NewSoloArgs) -> Self {
		NewSoloConfig { path: args.path }
	}
}

// endregion: --- NewConfig
