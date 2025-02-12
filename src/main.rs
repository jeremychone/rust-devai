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
use crate::hub::{get_hub, HubEvent};
use crate::tui::TuiApp;
use clap::{crate_version, Parser};
use error::{Error, Result};
use std::time::Duration;

pub static VERSION: &str = crate_version!();

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- Command arguments
	let args = CliArgs::parse(); // Will fail early, but that’s okay.

	// -- Start executor
	let mut executor = Executor::new();
	let executor_tx = executor.command_tx();
	// TODO: Probably want to move the spwn inside executor.start
	tokio::spawn(async move {
		if let Err(err) = executor.start().await {
			let hub = get_hub();
			hub.publish(HubEvent::Error { error: err.into() }).await;
			hub.publish(HubEvent::Quit).await;
		}
	});

	// -- Start UI
	let tui = TuiApp::new(executor_tx);
	// This will wait until all done
	tui.start_with_args(args).await?;

	// -- End
	// Tokio wait for 100ms
	// Note: This will allow the hub message to drain.
	//       This is a short-term trick before we get the whole TUI app.
	tokio::time::sleep(Duration::from_millis(100)).await;
	println!("\n     ---- Until next one, happy coding! ----");

	Ok(())
}
