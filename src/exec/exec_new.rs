use super::support::open_vscode;
use crate::cli::NewArgs;
use crate::hub::get_hub;
use crate::run::DirContext;
use crate::support::files::first_file_from_dirs;
use crate::Result;

/// exec for the New command
pub async fn exec_new(new_config: impl Into<NewConfig>, dir_context: DirContext) -> Result<()> {
	let hub = get_hub();
	let devai_dir = dir_context.devai_dir();

	let new_config = new_config.into();

	// TODO: support --template template_name
	let dirs = devai_dir.get_new_template_command_dirs()?;
	let dirs = dirs.iter().map(|dir| dir.to_str()).collect::<Vec<_>>();

	let template_file = first_file_from_dirs(
		&dirs,
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

	let dest_file = devai_dir.get_agent_custom_dir()?.join(file_path)?;

	if !dest_file.exists() {
		std::fs::copy(template_file.path(), &dest_file)?;
	}
	// if already exists, we publish the message, but we do not break
	else {
		hub.publish(format!(
			"-! Command agent file '{}' already exists.",
			dest_file.path().to_string_lossy()
		))
		.await;
	}

	if new_config.open {
		open_vscode(&dest_file).await;
	}

	Ok(())
}

// region:    --- NewConfig

#[derive(Debug)]
pub struct NewConfig {
	pub agent_path: String,

	/// If the file needs to be open (via code of vscode for now)
	pub open: bool,
}

impl From<NewArgs> for NewConfig {
	fn from(args: NewArgs) -> Self {
		NewConfig {
			agent_path: args.agent_path,
			open: args.open,
		}
	}
}

// endregion: --- NewConfig
