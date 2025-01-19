use crate::agent::{agent_agent_rel_as_bullet, list_all_agent_rels};
use crate::hub::get_hub;
use crate::run::DirContext;
use crate::Result;

pub async fn exec_list(dir_context: DirContext) -> Result<()> {
	let agent_rels = list_all_agent_rels(&dir_context)?;
	let msg = format!(
		"List of available command agents:\n{}",
		agent_rels
			.iter()
			.map(agent_agent_rel_as_bullet)
			.collect::<Vec<String>>()
			.join("\n")
	);
	get_hub().publish(msg).await;

	Ok(())
}
