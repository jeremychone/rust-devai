use super::support::open_vscode;
use crate::agent::{get_solo_and_target_path, load_solo_agent, Agent};
use crate::cli::SoloArgs;
use crate::exec::ExecEvent;
use crate::hub::get_hub;
use crate::run::{run_solo_agent, PathResolver, Runtime};
use crate::run::{DirContext, RunSoloOptions};
use crate::{Error, Result};
use simple_fs::{watch, SEventKind};
use std::sync::Arc;

pub struct SoloRedoCtx {
	runtime: Runtime,
	agent: Agent,
	solo_options: RunSoloOptions,
}

/// Executes the Run command
/// Can either perform a single run or run in watch mode
pub async fn exec_solo(solo_args: SoloArgs, dir_context: DirContext) -> Result<Arc<SoloRedoCtx>> {
	// -- First exec
	let redo_ctx: Arc<SoloRedoCtx> = exec_solo_first(solo_args, dir_context).await?.into();

	// -- If watch, we start the watch (will be spawned and return immediately)
	if redo_ctx.solo_options.base_run_config().watch() {
		exec_solo_watch(redo_ctx.clone());
	}

	Ok(redo_ctx)
}

pub async fn exec_solo_redo(solo_ctx: &SoloRedoCtx) {
	let hub = get_hub();
	let SoloRedoCtx {
		runtime,
		agent,
		solo_options,
	} = solo_ctx;

	// make sure to reload the agent
	let agent = match load_solo_agent(agent.file_path(), runtime.dir_context()) {
		Ok(agent) => agent,
		Err(err) => {
			hub.publish(err).await;
			return;
		}
	};

	match run_solo_agent(runtime, &agent, solo_options, PathResolver::CurrentDir).await {
		Ok(_) => (),
		Err(err) => hub.publish(Error::cc("Error while redo", err)).await,
	}
}

// region:    --- Privates

async fn exec_solo_first(solo_args: SoloArgs, dir_context: DirContext) -> Result<SoloRedoCtx> {
	let runtime = Runtime::new(dir_context)?;
	let (solo_path, target_path) = get_solo_and_target_path(&solo_args.path)?;
	let agent = load_solo_agent(solo_path.path(), runtime.dir_context())?;

	let solo_options = RunSoloOptions::new(solo_args, target_path)?;

	if solo_options.base_run_config().open() {
		open_vscode(agent.file_path()).await;
		open_vscode(solo_options.target_path()).await;
	}

	match run_solo_agent(&runtime, &agent, &solo_options, PathResolver::CurrentDir).await {
		Ok(_) => (),
		Err(err) => {
			get_hub()
				.publish(Error::cc(format!("Error while solo agent '{}'", agent.name()), err))
				.await
		}
	};

	Ok(SoloRedoCtx {
		runtime,
		agent,
		solo_options,
	})
}

fn exec_solo_watch(solo_ctx: Arc<SoloRedoCtx>) {
	tokio::spawn(async move {
		let watcher = match watch(solo_ctx.agent.file_path()) {
			Ok(watcher) => watcher,
			Err(err) => {
				get_hub().publish(Error::from(err)).await;
				return;
			}
		};

		loop {
			// Block until a message is received
			match watcher.rx.recv_async().await {
				Ok(events) => {
					// Process each event in the vector
					// TODO: Here we probably do not need to loop through the event, just check that there is at least one Modify
					for event in events {
						match event.skind {
							SEventKind::Modify => {
								let hub = get_hub();
								hub.publish("\n==== Agent file modified, running solo agent again\n").await;
								// Make sure to change reload the agent
								exec_solo_redo(&solo_ctx).await;
								// NOTE: here we trick the EndWatchRedo
								hub.publish(ExecEvent::EndWatchRedo).await;
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
	});
}

// endregion: --- Privates
