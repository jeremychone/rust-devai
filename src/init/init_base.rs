use crate::hub::get_hub;
use crate::run::paths::{CUSTOM_AGENT, CUSTOM_LUA, DEVAI_BASE};
use crate::Result;
use home::home_dir;
use simple_fs::ensure_dir;

pub async fn init_base() -> Result<()> {
	let hub = get_hub();

	// -- Check that the home dir exists
	let home_dir = home_dir().ok_or("No Home Dir Found, cannot init ./devai-base")?;
	if !home_dir.exists() {
		Err(format!("Home dir '{}' does not exist", home_dir.to_string_lossy()))?;
	}

	let mut changed = false;

	// -- Create the missing folders
	let base_dir = home_dir.join(DEVAI_BASE);
	if ensure_dir(&base_dir)? {
		changed = true;
		hub.publish(format!(
			"-> {:<35} '{}'",
			"Create ~/.devai-base dir at",
			base_dir.to_string_lossy()
		))
		.await;
	}

	let dirs = [CUSTOM_AGENT, CUSTOM_LUA];
	for dir_name in dirs {
		let sub_dir = base_dir.join(dir_name);
		if ensure_dir(&sub_dir)? {
			changed = true;
			hub.publish(format!(
				"-> {:<35} '{}'",
				format!("Create ~/.devai-base/{dir_name}"),
				sub_dir.to_string_lossy()
			))
			.await;
		}
	}

	// -- Display message
	if changed {
		hub.publish(format!(
			"-> {:<35} '{}'",
			"devai-base initialized at",
			base_dir.to_string_lossy()
		))
		.await;
	} else {
		hub.publish(format!(
			"-> {:<35} '{}'",
			"devai-base already initilized at",
			base_dir.to_string_lossy()
		))
		.await;
	}

	Ok(())
}
