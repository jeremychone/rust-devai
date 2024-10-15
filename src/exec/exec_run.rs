use crate::agent::{find_agent, Agent};
use crate::ai::{get_genai_client, run_command_agent};
use crate::exec::RunConfig;
use crate::hub::get_hub; // Importing get_hub
use crate::support::ValuesExt;
use crate::types::FileRef;
use crate::Result;
use genai::Client;
use simple_fs::{list_files, watch, SEventKind};

/// Exec for the Run command
/// Might do a single run or a watch
pub async fn exec_run(run_config: impl Into<RunConfig>) -> Result<()> {
	let run_config = run_config.into();

	// -- Get the AI client and agent
	let client = get_genai_client()?;
	let agent = find_agent(run_config.cmd_agent())?;
	let hub = get_hub();

	do_run(&run_config, &client, &agent).await?;

	if run_config.watch() {
		let watcher = watch(agent.file_path())?;
		// Continuously listen for events
		loop {
			// Block until a message is received
			match watcher.rx.recv() {
				Ok(events) => {
					// Process each event in the vector
					// TODO: Here we probably do not need to loop through the event, just check that there is at least one Modify
					for event in events {
						match event.skind {
							SEventKind::Modify => {
								hub.publish("\n==== Agent file modified, running agent again\n").await;
								// Make sure to change reload the agent
								let agent = find_agent(run_config.cmd_agent())?;

								match do_run(&run_config, &client, &agent).await {
									Ok(_) => (),
									Err(err) => hub.publish(format!("ERROR: {}", err)).await,
								}
								// Handle the modify event here
								// hub.publish(format!("File modified: {:?}", event.spath)).await; // Uncomment if needed
							}
							_ => {
								// Handle other event kinds if needed
								// hub.publish(format!("Other event: {:?}", event)).await; // Uncomment if needed
							}
						}
					}
				}
				Err(e) => {
					// Handle any errors related to receiving the message
					hub.publish(format!("Error receiving event: {:?}", e)).await;
					break;
				}
			}
		}
	}

	Ok(())
}

/// Do one run
async fn do_run(run_config: &RunConfig, client: &Client, agent: &Agent) -> Result<()> {
	// -- Execute the command
	let on_file_globs = run_config.on_file_globs();
	// If we have the on_file_globs, they become the items
	if let Some(on_file_globs) = on_file_globs {
		let files = list_files("./", Some(&on_file_globs), None)?;
		let file_refs = files.into_iter().map(FileRef::from).collect::<Vec<_>>();
		run_command_agent(client, agent, Some(file_refs.x_into_values()?), run_config.into()).await?;
	} else {
		run_command_agent(client, agent, None, run_config.into()).await?;
	}

	Ok(())
}
