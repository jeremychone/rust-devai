// region:    --- Modules

mod agent;
mod cli;
mod error;
mod hub;
mod init;
mod run;
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
		cli::Commands::Init(init_args) => {
			init_devai_files(init_args.path).await?;
		}

		// Create a new agent
		cli::Commands::New(new_args) => {
			cli::exec_new(new_args, init_devai_files(None).await?).await?;
		}

		// Run an agent command
		cli::Commands::Run(run_args) => {
			// Note: Every run will initialize the files
			// Execute the command
			cli::exec_run(run_args, init_devai_files(None).await?).await?;
		}

		// Create a new agent
		cli::Commands::NewSolo(new_args) => {
			cli::exec_new_solo(new_args, init_devai_files(None).await?).await?;
		}

		// Run a solo agent
		cli::Commands::Solo(solo_args) => {
			// Execute the command
			cli::exec_solo(solo_args, init_devai_files(None).await?).await?;
		}

		// List the available agents
		cli::Commands::List => {
			cli::exec_list(init_devai_files(None).await?).await?;
		}
	}

	Ok(())
}
