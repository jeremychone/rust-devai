use crate::cli::PackArgs;
use crate::hub::get_hub;
use crate::pack::pack_dir;
use crate::{Error, Result};
use camino::Utf8PathBuf;
use std::path::Path;

/// Execute the pack command which creates a .aipack file from a directory
pub async fn exec_pack(pack_args: &PackArgs) -> Result<()> {
	let hub = get_hub();

	// Get source directory path
	let src_dir = Utf8PathBuf::from(&pack_args.dir_path);
	if !src_dir.exists() {
		return Err(Error::custom(format!("Source directory '{}' does not exist", src_dir)));
	}

	// Get destination directory (default to current directory if not specified)
	let dest_dir = if let Some(output_dir) = &pack_args.output_dir {
		Utf8PathBuf::from(output_dir)
	} else {
		Utf8PathBuf::from(".")
	};

	// Create output directory if it doesn't exist
	if !dest_dir.exists() {
		std::fs::create_dir_all(&dest_dir)?;
	}

	// Perform the packing
	hub.publish(format!("Packing directory '{}' into a .aipack file...", src_dir))
		.await;

	match pack_dir(&src_dir, &dest_dir) {
		Ok(pack_data) => {
			hub.publish(format!("Successfully packed directory into '{}'", pack_data.pack_file))
				.await;
			Ok(())
		}
		Err(err) => {
			hub.publish(format!("Failed to pack directory: {}", err)).await;
			Ok(())
		}
	}
}
