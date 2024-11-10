use super::support::open_vscode;
use crate::agent::{get_solo_and_target_path, load_solo_agent};
use crate::cli::SoloArgs;
use crate::hub::get_hub;
use crate::run::{run_solo_agent, PathResolver, Runtime};
use crate::run::{DirContext, RunSoloOptions};
use crate::Result;
use simple_fs::{watch, SEventKind};

/// Executes the Run command
/// Can either perform a single run or run in watch mode
pub async fn exec_solo(solo_args: SoloArgs, dir_context: DirContext) -> Result<()> {
	// -- Get the AI client and agent
	let hub = get_hub();

	let runtime = Runtime::new(dir_context)?;
	let (solo_path, target_path) = get_solo_and_target_path(&solo_args.path)?;
	let agent = load_solo_agent(solo_path.path(), runtime.dir_context())?;
	let solo_options = RunSoloOptions::new(solo_args, target_path)?;

	if solo_options.base_run_config().open() {
		open_vscode(agent.file_path()).await;
		open_vscode(solo_options.target_path()).await;
	}

	// -- If NOT in watch mode, then just run once
	if !solo_options.base_run_config().watch() {
		run_solo_agent(&runtime, &agent, &solo_options, PathResolver::CurrentDir).await?;
	}
	// -- If in watch mode
	else {
		// Do the first run
		let agent = load_solo_agent(agent.file_path(), runtime.dir_context())?;
		match run_solo_agent(&runtime, &agent, &solo_options, PathResolver::CurrentDir).await {
			Ok(_) => (),
			Err(err) => hub.publish(format!("ERROR: {}", err)).await,
		}

		// And watch for modifications
		let watcher = watch(agent.file_path())?;
		loop {
			match watcher.rx.recv() {
				Ok(events) => {
					// If there is a modification, then run again
					let has_modify = events.iter().any(|evt| matches!(evt.skind, SEventKind::Modify));
					if has_modify {
						hub.publish(format!("\nSolo Agent Modified '{}', running again.", agent.file_path()))
							.await;
						// Ensure to reload the agent
						let agent = load_solo_agent(agent.file_path(), runtime.dir_context())?;

						match run_solo_agent(&runtime, &agent, &solo_options, PathResolver::CurrentDir).await {
							Ok(_) => (),
							Err(err) => hub.publish(format!("ERROR: {}", err)).await,
						}
					}
				}
				Err(err) => {
					// Handle any errors related to receiving the message
					hub.publish(format!("Error receiving event: {:?}", err)).await;
					break;
				}
			}
		}
	}

	Ok(())
}
