use crate::Result;
use crate::cli::InstallArgs;
use crate::dir_context::DirContext;
use crate::hub::get_hub;
use simple_fs::SPath;

// region:    --- InstallRef

enum InstallRef {
	/// Direct path
	Path(SPath),
	// Future the PackRef path `namespace@pack_name`
}

// endregion: --- InstallRef

/// Executes the install command which installs an aipack file
pub async fn exec_install(dir_context: DirContext, install_args: InstallArgs) -> Result<()> {
	let hub = get_hub();
	hub.publish(format!("\nInstalling aipack from '{}'...", install_args.aipack_ref))
		.await;

	Ok(())
}
