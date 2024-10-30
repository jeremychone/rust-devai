use crate::agent::{find_agent, Agent};
use crate::ai::{get_genai_client, run_command_agent};
use crate::exec::support::open_vscode;
use crate::hub::get_hub; // Importing get_hub
use crate::support::RunCommandOptions;
use crate::support::ValuesExt;
use crate::types::FileRef;
use crate::Result;
use genai::Client;
use simple_fs::{list_files, watch, SEventKind};
use std::io::{self, Read}; // Importing io and Read
use serde_json::Value;

/// Exec for the Run command
/// Might do a single run or a watch
pub async fn exec_run(run_command_options: impl Into<RunCommandOptions>) -> Result<()> {
	let hub = get_hub();

	let run_options: RunCommandOptions = run_command_options.into();

	// -- Get the AI client and agent
	let client = get_genai_client()?;
	let agent = find_agent(run_options.cmd_agent())?;

	if run_options.base_run_config().open() {
		open_vscode(agent.file_path()).await;
	}

	do_run(&run_options, &client, &agent).await?;

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
								let agent = find_agent(run_options.cmd_agent())?;

								match do_run(&run_options, &client, &agent).await {
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
async fn do_run(run_command_options: &RunCommandOptions, client: &Client, agent: &Agent) -> Result<()> {
	let on_file_globs = run_command_options.on_file_globs();
	let file_refs = if let Some(on_file_globs) = on_file_globs {
		let files = list_files("./", Some(&on_file_globs), None)?;
		Some(files.into_iter().map(FileRef::from).collect::<Vec<_>>().x_into_values()?)
	} else {
		 // Read from stdin if no file globs are provided
		let stdin_content = read_stdin()?;
		Some(vec![stdin_content])
	};

	run_command_agent(client, agent, file_refs, run_command_options.base_run_config()).await?;

	Ok(())
}

/// Helper function to read from stdin and return a Value
fn read_stdin() -> Result<Value> {
	let mut buffer = String::new();
	io::stdin().read_to_string(&mut buffer)?;
	Ok(Value::String(buffer))
}
