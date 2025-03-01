use crate::cli::PackArgs;
use crate::hub::get_hub;
use crate::init::extract_template_pack_toml_zfile;
use crate::packer::pack_dir;
use crate::{Error, Result};
use aho_corasick::AhoCorasick;
use camino::Utf8PathBuf;
use std::fs;
use std::io::{self};

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
	hub.publish(format!("\nPacking directory '{}' into a .aipack file...", src_dir))
		.await;

	match pack_dir(&src_dir, &dest_dir) {
		Ok(pack_data) => {
			hub.publish(format!(
				"\nSuccessfully packed directory into '{}'",
				pack_data.pack_file
			))
			.await;
			Ok(())
		}
		Err(Error::AipackTomlMissing(_missing_toml_path)) => {
			// Generate template pack.toml
			if let Err(gen_err) = generate_pack_toml(&src_dir).await {
				hub.publish(format!("Failed to generate pack.toml: {}", gen_err)).await;
				return Ok(());
			}

			// Prompt user to continue
			hub.publish(format!(
				"\n{src_dir}/pack.toml was missing, a default one was generated\n\nPlease check {}/pack.toml",
				src_dir
			))
			.await;

			// Wait for user input
			hub.publish("\nContinue with packing? (Y/n): ").await;

			// Workaround for now. Need to investigate how we can remove this.
			// tokio yield
			tokio::task::yield_now().await;
			// tokio wait 10ms
			tokio::time::sleep(std::time::Duration::from_millis(10)).await;

			let mut input = String::new();
			io::stdin().read_line(&mut input)?;

			if input.trim().to_uppercase() == "Y" {
				// Try packing again
				match pack_dir(&src_dir, &dest_dir) {
					Ok(pack_data) => {
						hub.publish(format!("Successfully packed directory into '{}'", pack_data.pack_file))
							.await;
					}
					Err(retry_err) => {
						hub.publish(format!(
							"Failed to pack directory after generating pack.toml: {}",
							retry_err
						))
						.await;
					}
				}
			} else {
				hub.publish("\nPacking cancelled by user.").await;
			}

			Ok(())
		}
		Err(other) => Err(other),
	}
}

/// Generates a default pack.toml file from the template
async fn generate_pack_toml(dir_path: &Utf8PathBuf) -> Result<()> {
	let hub = get_hub();

	// Extract the template pack.toml
	let pack_toml_zfile = extract_template_pack_toml_zfile()?;
	let mut content =
		String::from_utf8(pack_toml_zfile.content).map_err(|_| Error::custom("template pack.toml is not UTF8 ??"))?;

	// Get the directory name
	let dir_name = dir_path
		.file_name()
		.ok_or_else(|| Error::custom("Unable to extract directory name"))?;

	// Replace DIR_NAME with actual directory name using aho-corasick
	let patterns = &["DIR_NAME"];
	let replacements = &[dir_name];
	let ac =
		AhoCorasick::new(patterns).map_err(|err| Error::custom(format!("AhoCorasick pattern fail. Cause: {err}")))?;
	content = ac.replace_all(&content, replacements);

	// Write the file
	let toml_path = dir_path.join("pack.toml");
	fs::write(&toml_path, content)?;

	hub.publish(format!("-> Created default pack.toml at '{}'", toml_path)).await;

	Ok(())
}
