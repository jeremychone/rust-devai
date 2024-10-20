use crate::agent::{load_base_agent_config, Agent, AgentDoc};
use crate::ai::{get_genai_client, run_solo_agent};
use crate::exec::support::open_vscode;
use crate::hub::get_hub;
use crate::support::RunSoloOptions;
use crate::{Error, Result};
use simple_fs::{watch, SEventKind, SFile};

/// Executes the Run command
/// Can either perform a single run or run in watch mode
pub async fn exec_solo<T>(run_solo_options: T) -> Result<()>
where
	T: TryInto<RunSoloOptions, Error = Error>,
{
	let run_solo_options: RunSoloOptions = run_solo_options.try_into()?;

	// -- Get the AI client and agent
	let client = get_genai_client()?;
	let hub = get_hub();

	if run_solo_options.base_run_config().open() {
		open_vscode(run_solo_options.solo_path()).await;
		open_vscode(run_solo_options.target_path()).await;
	}

	// -- If NOT in watch mode, then just run once
	if !run_solo_options.base_run_config().watch() {
		let agent = load_solo_agent(&run_solo_options)?;
		run_solo_agent(&client, &agent, &run_solo_options).await?;
	}
	// -- If in watch mode
	else {
		// Do the first run
		let agent = load_solo_agent(&run_solo_options)?;
		match run_solo_agent(&client, &agent, &run_solo_options).await {
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
						hub.publish(format!(
							"\nSolo Agent Modified '{}', running again.",
							run_solo_options.solo_path()
						))
						.await;
						// Ensure to reload the agent
						let agent = load_solo_agent(&run_solo_options)?;

						match run_solo_agent(&client, &agent, &run_solo_options).await {
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

// region:    --- Support

fn load_solo_agent(run_solo_options: &RunSoloOptions) -> Result<Agent> {
	// TODO: Create it if solo_config.create_if_needed with the eventual template

	let solo_file =
		SFile::new(run_solo_options.solo_path().path()).map_err(|err| format!("Solo file not found: {err}"))?;
	let base_config = load_base_agent_config()?;

	let agent_doc = AgentDoc::from_file(solo_file)?;
	agent_doc.into_agent(base_config)
}

// endregion: --- Support
