// region:    --- Modules

mod agent;
mod cli;
mod error;
mod exec;
mod hub;
mod init;
mod run;
mod script;
mod support;
mod tui;
mod types;

#[cfg(test)]
mod _test_support;

use crate::cli::CliArgs;
use crate::exec::Executor;
use crate::tui::TuiApp;
use clap::Parser;
use error::{Error, Result};
use std::time::Duration;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- Command arguments
	let args = CliArgs::parse(); // will fail early, but that’s okay.

	// -- Start executor
	let mut executor = Executor::new();
	let executor_tx = executor.command_tx();
	// TODO: todo probably want to move the spwn inside executor.start
	tokio::spawn(async move {
		if let Err(err) = executor.start().await {
			println!("Error starting the executor - {err}");
		}
	});

	// -- Start UI
	let tui = TuiApp::new(executor_tx);
	// This will wait all done
	tui.start_with_args(args).await?;

	// -- End
	// tokio wait for 100ms
	// Note: This will allow the hub message to drain
	//       This is a shorterm trick before we get the whole TUI app
	tokio::time::sleep(Duration::from_millis(100)).await;
	println!("\n     ---- Until next one, happy coding! ----");

	Ok(())
}
