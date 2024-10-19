use crate::cli::NewArgs;
use crate::exec::support::first_file_from_dirs;
use crate::init::{DEVAI_AGENT_CUSTOM_DIR, DEVAI_NEW_CUSTOM_COMMAND_AGENT_DIR, DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR};
use crate::Result;
use std::path::Path;

/// exec for the New command
pub async fn exec_new(new_config: impl Into<NewConfig>) -> Result<()> {
	let new_config = new_config.into();

	// TODO: support --template template_name
	let template_file = first_file_from_dirs(
		&[DEVAI_NEW_CUSTOM_COMMAND_AGENT_DIR, DEVAI_NEW_DEFAULT_COMMAND_AGENT_DIR],
		"default.devai", // for now, just look for default.devai
	)
	.ok()
	.flatten()
	.ok_or("command agent template 'default.devai' not found")?;

	let file_path = if new_config.agent_path.ends_with(".devai") {
		new_config.agent_path
	} else {
		format!("{}.devai", new_config.agent_path)
	};

	let dest_file = Path::new(DEVAI_AGENT_CUSTOM_DIR).join(file_path);

	if dest_file.exists() {
		return Err(format!("Command agent file '{}' already exists.", dest_file.to_string_lossy()).into());
	}

	std::fs::copy(template_file.path(), dest_file)?;

	Ok(())
}

// region:    --- NewConfig

#[derive(Debug)]
pub struct NewConfig {
	pub agent_path: String,
}

impl From<NewArgs> for NewConfig {
	fn from(args: NewArgs) -> Self {
		NewConfig {
			agent_path: args.agent_path,
		}
	}
}

// endregion: --- NewConfig
