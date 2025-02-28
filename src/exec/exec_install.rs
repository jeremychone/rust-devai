use crate::Result;
use crate::cli::InstallArgs;
use crate::dir_context::DirContext;
use crate::hub::get_hub;
use crate::packer::install_pack;

// region:    --- InstallRef

// endregion: --- InstallRef

/// Executes the install command which installs an aipack file
pub async fn exec_install(dir_context: DirContext, install_args: InstallArgs) -> Result<()> {
	let hub = get_hub();
	hub.publish(format!(
		"\n==== Installing aipack from '{}'...",
		install_args.aipack_ref
	))
	.await;

	install_pack(&dir_context, &install_args.aipack_ref)?;

	hub.publish("\n==== DONE".to_string()).await;

	Ok(())
}
