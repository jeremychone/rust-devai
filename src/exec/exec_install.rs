use crate::Result;
use crate::cli::InstallArgs;
use crate::dir_context::DirContext;
use crate::hub::get_hub;
use crate::packer::install_pack;
use size::Size;

// region:    --- InstallRef

// endregion: --- InstallRef

/// Executes the install command which installs an aipack file
pub async fn exec_install(dir_context: DirContext, install_args: InstallArgs) -> Result<()> {
	let hub = get_hub();
	hub.publish(format!(
		"\n==== Installing aipack:\n\n{:>15} {}",
		"From:", install_args.aipack_ref
	))
	.await;

	let installed_pack = install_pack(&dir_context, &install_args.aipack_ref).await?;

	// Format the zip size using the size crate
	let formatted_zip_size = Size::from_bytes(installed_pack.zip_size as u64).to_string();

	// Format the size using the size crate
	// let formatted_size = Size::from_bytes(installed_pack.size as u64).to_string();

	hub.publish(format!(
		"{:>15} {}\n{:>15} {}@{}\n{:>15} {}\n{:>15} {}",
		".aipack Size:",
		formatted_zip_size,
		"Pack:",
		installed_pack.pack_toml.namespace,
		installed_pack.pack_toml.name,
		"Version:",
		installed_pack.pack_toml.version,
		"Installed At:",
		installed_pack.path
	))
	.await;

	hub.publish("\n==== DONE".to_string()).await;

	Ok(())
}
