use crate::dir_context::DirContext;
use crate::packer::pack_toml::parse_validate_pack_toml;
use crate::support::zip;
use crate::{Error, Result};
use simple_fs::SPath;

/// Install a `file.aipack` into the .aipack-base/pack/installed directory
///
/// IMPORTANT: Right now, very prelimealy. Should do the following:
///
/// TODO:
/// - Check for an existing installed pack.
/// - If an already installed pack has a semver greater than the new one,
///   return an error so that the caller can handle it with a prompt, and then provide a force flag, for example.
/// - Probably need to remove the existing pack files; otherwise, some leftover files can be an issue.
///
/// Returns the SPath directory where this zip was unzipped.
pub fn install_pack(dir_context: &DirContext, aipack_zipped_file: &str) -> Result<SPath> {
	// -- Parse the pack_to_install_path
	// for now, only support .aipack file (later will support aipack.ai remote name some@pack_name)
	let aipack_zipped_file = SPath::from(aipack_zipped_file);
	let aipack_zipped_file = if aipack_zipped_file.path().is_absolute() {
		aipack_zipped_file
	} else {
		dir_context.current_dir().join_str(aipack_zipped_file.to_str())
	};

	println!("->> {aipack_zipped_file}");

	if !aipack_zipped_file.exists() {
		return Err(Error::FailToInstall {
			aipack_file: aipack_zipped_file.to_string(),
			cause: "aipack file does not exists".to_string(),
		});
	}
	if aipack_zipped_file.ext() != "aipack" {
		return Err(Error::FailToInstall {
			aipack_file: aipack_zipped_file.to_string(),
			cause: format!(
				"aipack file must be '.aipack' file, but was {}",
				aipack_zipped_file.name()
			),
		});
	}

	// -- Get the aipack base pack install dir
	// This is the pack base dir and now, we need ot add `namespace/pack_name`
	let pack_installed_dir = dir_context.aipack_paths().get_base_pack_installed_dir()?;

	if !pack_installed_dir.exists() {
		return Err(Error::FailToInstall {
			aipack_file: aipack_zipped_file.to_string(),
			cause: format!(
				"aipack base directory '{pack_installed_dir}' not found.\n   recommendation: Run 'aip init'"
			),
		});
	}

	// -- Extract the pack.toml from zip
	let toml_content = zip::extract_text_content(&aipack_zipped_file, "pack.toml")?;
	let pack_toml = parse_validate_pack_toml(&toml_content, &format!("{aipack_zipped_file} pack.toml"))?;

	let pack_target_dir = pack_installed_dir.join_str(&pack_toml.namespace).join_str(&pack_toml.name);

	zip::unzip_file(aipack_zipped_file, &pack_target_dir)?;

	Ok(pack_target_dir)
}

// region:    --- Tests

// TODO: Need to write basic test about this one

// endregion: --- Tests
