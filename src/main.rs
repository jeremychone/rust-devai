// region:    --- Modules

mod agent;
mod ai;
mod cli;
mod error;
mod exec;
mod script;
mod support;
mod types;

use crate::agent::init_agent_files;
use crate::cli::AppArgs;
use clap::Parser;
pub use error::{Error, Result};

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- Command arguments
	let args = AppArgs::parse(); // will fail early, but that’s okay.

	// -- match the run
	match args.cmd {
		// Run an agent command
		cli::Commands::Run(run_args) => {
			// Note: Every run will init the files
			init_agent_files()?;
			// exec the command
			exec::exec_run(run_args).await?;
		}
		// Init devai for this folder
		cli::Commands::Init => {
			init_agent_files()?;
		}
	}

	Ok(())
}
