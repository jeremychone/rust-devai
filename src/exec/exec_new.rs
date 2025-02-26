use crate::Result;
use crate::cli::NewArgs;
use crate::dir_context::DirContext;
use crate::hub::get_hub;

/// exec for the New command
/// Will create a new pack in base or workspace custom (not sure yet)
///
/// NOTE: THIS IS DISABLED FOR NOW
pub async fn exec_new(new_config: impl Into<NewConfig>, dir_context: DirContext) -> Result<()> {
	let _hub = get_hub();
	let _aipack_paths = dir_context.aipack_paths();

	let _new_config = new_config.into();

	todo!("This needs to be reimplemented")
	// Ok(())
}

// region:    --- NewConfig

#[allow(unused)]
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
