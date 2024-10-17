// region:    --- Modules

mod agent;
mod ai;
mod cli;
mod error;
mod exec;
mod hub;
mod init;
mod script;
mod support;
mod tui;
mod types;

#[cfg(test)]
mod _test_support;

use crate::cli::AppArgs;
use crate::hub::get_hub;
use crate::init::init_devai_files;
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

	// Note: No need to print the error; the TUI will handle that.
	match main_inner(args).await {
		Ok(_) => (),
		Err(err) => get_hub().publish(err).await,
	};

	// A temporary measure to ensure that the eventual error(s) get printed.
	// TODO: Need to make this code more sound. Perhaps a .close.
	tokio::time::sleep(std::time::Duration::from_millis(100)).await;

	Ok(())
}

async fn main_inner(args: AppArgs) -> Result<()> {
	// -- Match the run
	match args.cmd {
		// Initialize the device for this folder
		cli::Commands::Init => {
			init_devai_files()?;
		}

		// Run an agent command
		cli::Commands::Run(run_args) => {
			// Note: Every run will initialize the files
			init_devai_files()?;
			// Execute the command
			exec::exec_run(run_args).await?;
		}

		// Run a solo agent
		cli::Commands::Solo(solo_args) => {
			// Note: Every run will initialize the files
			init_devai_files()?;
			// Execute the command
			exec::exec_solo(solo_args).await?;
		}

		// List the available agents
		cli::Commands::List => {
			init_devai_files()?;
			exec::exec_list().await?;
		}

		// Create a new agent
		cli::Commands::New(new_args) => {
			init_devai_files()?;
			exec::exec_new(new_args).await?;
		}
	}

	Ok(())
}
