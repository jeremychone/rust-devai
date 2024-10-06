// region:    --- Modules

mod agent;
mod ai;
mod cli;
mod error;
mod exec;
mod hub;
mod script;
mod support;
mod tui;
mod types;

#[cfg(test)]
mod test_support;

use crate::agent::init_agent_files;
use crate::cli::AppArgs;
use crate::hub::get_hub;
use crate::tui::Tui;
use clap::Parser;
use error::{Error, Result};

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- Command arguments
	let args = AppArgs::parse(); // will fail early, but that’s okay.
	let tui = Tui;
	tui.start_printer()?;

	// Note: No need to print the error, the TUI will handle that
	match main_inner(args).await {
		Ok(_) => (),
		Err(err) => get_hub().publish(err).await,
	};

	// Hack for now, to make sure the eventual error(s) get printed.
	// TODO: Need to make this code more sound. Perhaps a .close.
	tokio::time::sleep(std::time::Duration::from_millis(100)).await;

	Ok(())
}

async fn main_inner(args: AppArgs) -> Result<()> {
	// -- match the run
	match args.cmd {
		// Run an agent command
		cli::Commands::Run(run_args) => {
			// Note: Every run will initialize the files
			init_agent_files()?;
			// Execute the command
			exec::exec_run(run_args).await?;
		}

		// Initialize the device for this folder
		cli::Commands::Init => {
			init_agent_files()?;
		}

		// List the available agents
		cli::Commands::List => {
			init_agent_files()?;
			exec::exec_list().await?;
		}
	}

	Ok(())
}
