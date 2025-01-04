use crate::hub::get_hub;
use crate::run::DevaiDir;
use crate::Result;
use simple_fs::SPath;
use std::fs;

pub async fn migrate_devai_0_5_0_if_needed(devai_dir: &DevaiDir) -> Result<bool> {
	let mut changed = false;

	// -- custom/command-agent -> custom/agent
	let new_dir = devai_dir.get_custom_agent_dir()?;
	changed |= migrate_dir(devai_dir, "custom/command-agent", &new_dir).await?;

	// -- default/command-agent -> default/agent
	let new_dir = devai_dir.get_default_agent_dir()?;
	changed |= migrate_dir(devai_dir, "default/command-agent", &new_dir).await?;

	// -- custom/new-template/command-agent -> custom/new-template/agent
	let new_dir = devai_dir.get_custom_new_template_dir()?;
	changed |= migrate_dir(devai_dir, "custom/new-template/command-agent", &new_dir).await?;

	// -- default/new-template/command-agent -> default/new-template/agent
	let new_dir = devai_dir.get_default_new_template_dir()?;
	changed |= migrate_dir(devai_dir, "default/new-template/command-agent", &new_dir).await?;

	Ok(changed)
}

async fn migrate_dir(devai_dir: &DevaiDir, from_dir_str: &str, to_dir: &SPath) -> Result<bool> {
	let hub = get_hub();

	let devai_dir_path = devai_dir.devai_dir_full_path();
	let workspace_dir = devai_dir.workspace_dir();

	let Ok(from_dir) = devai_dir_path.join(from_dir_str) else {
		return Ok(false);
	};

	if !from_dir.exists() {
		return Ok(false);
	}

	let mut changed = false;

	if !to_dir.exists() {
		fs::rename(&from_dir, to_dir)?;
		changed = true;
		hub.publish(format!(
			"-> {:<18} from '{}' to '{}'",
			".devai/ content migration.",
			from_dir.diff(workspace_dir)?,
			to_dir.diff(workspace_dir)?
		))
		.await;
	} else {
		hub.publish(format!(
			"-> {:<18} from '{}' to '{}'",
			".devai/ content migration skipped (already exist).",
			from_dir.diff(workspace_dir)?,
			to_dir.diff(workspace_dir)?
		))
		.await;
	}

	Ok(changed)
}
