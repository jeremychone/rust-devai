use crate::agent::find_agent;
use crate::ai::{get_genai_client, run_agent_items};
use crate::exec::RunConfig;
use crate::support::ValuesExt;
use crate::types::FileRef;
use crate::Result;
use simple_fs::list_files;

pub async fn exec_run(run_config: impl Into<RunConfig>) -> Result<()> {
	let run_config = run_config.into();

	// -- Get the AI client and agent
	let client = get_genai_client()?;
	let agent = find_agent(run_config.cmd_agent())?;

	// -- Execute the command
	let on_file_globs = run_config.on_file_globs();
	// If we have the on_file_globs, they become the items
	if let Some(on_file_globs) = on_file_globs {
		let files = list_files("./", Some(&on_file_globs), None)?;
		let file_refs = files.into_iter().map(FileRef::from).collect::<Vec<_>>();
		run_agent_items(client, agent, Some(file_refs.x_into_values()?)).await?;
	} else {
		run_agent_items(client, agent, None).await?;
	}

	Ok(())
}
