use super::support::open_vscode;
use crate::agent::{find_agent, Agent};
use crate::cli::RunArgs;
use crate::hub::get_hub; // Importing get_hub
use crate::run::{run_command_agent, PathResolver, Runtime};
use crate::run::{DirContext, RunCommandOptions};
use crate::support::jsons::into_values;
use crate::types::FileRef;
use crate::Result;
use simple_fs::{list_files, watch, SEventKind};

pub struct RunRedoCtx {
	runtime: Runtime,
	agent: Agent,
	run_options: RunCommandOptions,
}

pub async fn exec_run_first(run_args: RunArgs, dir_context: DirContext) -> Result<RunRedoCtx> {
	let hub = get_hub();

	let cmd_agent_name = &run_args.cmd_agent_name;

	let runtime = Runtime::new(dir_context)?;
	let agent = find_agent(cmd_agent_name, runtime.dir_context(), PathResolver::CurrentDir)?;

	let run_options = RunCommandOptions::new(run_args)?;

	if run_options.base_run_config().open() {
		open_vscode(agent.file_path()).await;
	}

	match do_run(&run_options, &runtime, &agent).await {
		Ok(_) => (),
		Err(err) => hub.publish(format!("ERROR: {}", err)).await,
	};

	Ok(RunRedoCtx {
		runtime,
		agent,
		run_options,
	})
}

pub async fn exec_run_redo(run_redo_ctx: &RunRedoCtx) -> Result<()> {
	let hub = get_hub();

	let RunRedoCtx {
		runtime,
		agent,
		run_options,
	} = run_redo_ctx;

	// make sure to reload the agent
	let agent = find_agent(agent.name(), runtime.dir_context(), PathResolver::CurrentDir)?;

	match do_run(run_options, runtime, &agent).await {
		Ok(_) => (),
		Err(err) => hub.publish(format!("ERROR: {}", err)).await,
	};

	Ok(())
}

/// Exec for the Run command
/// Might do a single run or a watch
pub async fn exec_run(run_args: RunArgs, dir_context: DirContext) -> Result<()> {
	let redo_ctx = exec_run_first(run_args, dir_context).await?;

	if redo_ctx.run_options.base_run_config().watch() {
		let watcher = watch(redo_ctx.agent.file_path())?;

		loop {
			// Block until a message is received
			match watcher.rx.recv_async().await {
				Ok(events) => {
					// Process each event in the vector
					// TODO: Here we probably do not need to loop through the event, just check that there is at least one Modify
					for event in events {
						match event.skind {
							SEventKind::Modify => {
								get_hub().publish("\n==== Agent file modified, running agent again\n").await;
								// Make sure to change reload the agent
								exec_run_redo(&redo_ctx).await?;
								// NOTE: No need to notify for now
							}
							_ => {
								// NOTE: No need to notify for now
							}
						}
					}
				}
				Err(e) => {
					// Handle any errors related to receiving the message
					get_hub().publish(format!("Error receiving event: {:?}", e)).await;
					break;
				}
			}
		}
	}

	Ok(())
}

/// Do one run
async fn do_run(run_command_options: &RunCommandOptions, runtime: &Runtime, agent: &Agent) -> Result<()> {
	let inputs = if let Some(on_inputs) = run_command_options.on_inputs() {
		Some(into_values(on_inputs)?)
	} else if let Some(on_file_globs) = run_command_options.on_file_globs() {
		let files = list_files("./", Some(&on_file_globs), None)?;
		let file_refs = files.into_iter().map(FileRef::from).collect::<Vec<_>>();
		Some(into_values(file_refs)?)
	} else {
		None
	};

	run_command_agent(runtime, agent, inputs, run_command_options.base_run_config(), false).await?;

	Ok(())
}
