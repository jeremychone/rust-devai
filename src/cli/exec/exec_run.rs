use super::support::open_vscode;
use crate::agent::{find_agent, Agent};
use crate::cli::RunArgs;
use crate::hub::get_hub; // Importing get_hub
use crate::run::{run_command_agent, Runtime};
use crate::run::{DirContext, RunCommandOptions};
use crate::support::jsons::into_values;
use crate::types::FileRef;
use crate::Result;
use simple_fs::{list_files, watch, SEventKind};

/// Exec for the Run command
/// Might do a single run or a watch
pub async fn exec_run(run_args: RunArgs, dir_context: DirContext) -> Result<()> {
	let hub = get_hub();

	let cmd_agent_name = &run_args.cmd_agent_name;

	// -- Get the AI client and agent
	let runtime = Runtime::new(dir_context)?;
	let agent = find_agent(cmd_agent_name, runtime.dir_context())?;

	let run_options = RunCommandOptions::new(run_args)?;

	if run_options.base_run_config().open() {
		open_vscode(agent.file_path()).await;
	}

	do_run(&run_options, &runtime, &agent).await?;

	if run_options.base_run_config().watch() {
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
								let agent = find_agent(agent.file_path(), runtime.dir_context())?;

								match do_run(&run_options, &runtime, &agent).await {
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
async fn do_run(run_command_options: &RunCommandOptions, runtime: &Runtime, agent: &Agent) -> Result<()> {
	let on_file_globs = run_command_options.on_file_globs();
	let file_refs = if let Some(on_file_globs) = on_file_globs {
		let files = list_files("./", Some(&on_file_globs), None)?;
		let file_refs = files.into_iter().map(FileRef::from).collect::<Vec<_>>();
		Some(into_values(file_refs)?)
	} else {
		None
	};

	run_command_agent(runtime, agent, file_refs, run_command_options.base_run_config(), false).await?;

	Ok(())
}
