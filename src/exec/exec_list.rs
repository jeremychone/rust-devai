use crate::agent::{agent_sfile_as_bullet, list_all_agent_files};
use crate::hub::get_hub;
use crate::Result;

pub async fn exec_list() -> Result<()> {
	let agent_files = list_all_agent_files()?;
	let msg = format!(
		"List of available command agents:\n{}",
		agent_files
			.iter()
			.map(agent_sfile_as_bullet)
			.collect::<Vec<String>>()
			.join("\n")
	);
	get_hub().publish(msg).await;

	Ok(())
}
