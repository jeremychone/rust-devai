use crate::cli::{self, CliArgs};
use crate::exec::{ExecCommand, ExecEvent};
use crate::hub::{get_hub, HubEvent};
use crate::init::init_devai_files;
use crate::Result;
use std::io::Write as _;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{mpsc, oneshot, watch};

/// Note: Right now the quick channel is a watch, but might be better to be a mpsc.
#[derive(Debug)]
pub struct Tui {
	executor_tx: mpsc::Sender<ExecCommand>,
}

/// Constructor
impl Tui {
	pub fn new(executor_tx: mpsc::Sender<ExecCommand>) -> Self {
		Self { executor_tx }
	}
}

/// Getters
impl Tui {
	fn executor_tx(&self) -> mpsc::Sender<ExecCommand> {
		self.executor_tx.clone()
	}
}

/// Starter
impl Tui {
	pub async fn start_with_args(self, cli_args: CliArgs) -> Result<()> {
		// Make sure to subscribe early to have all events
		let mut hub_rx = get_hub().subscriber();

		// -- Start the printer (very rudementary "cli UI for now")
		self.start_printer();

		// -- Exec the first cli_args
		// NOTE: for now, we wait unitl this exec is done.
		let done_rx = self.exec_cli_args(cli_args)?;
		done_rx.await;

		self.wait_for_quit(hub_rx).await?;

		// tokio::time::sleep(Duration::from_secs(10)).await;

		Ok(())
	}

	/// Very rundemetary printer
	/// Note: This function is designed to spawn it's on work and return, so that it does not block the async caller.
	fn start_printer(&self) -> Result<()> {
		let mut rx = get_hub().subscriber();

		tokio::spawn(async move {
			while let Ok(event) = rx.recv().await {
				let mut stdout = tokio::io::stdout();

				match event {
					HubEvent::Message(msg) => {
						stdout.write_all(format!("{msg}\n").as_bytes()).await;
					}
					HubEvent::Error { error } => {
						stdout.write_all(format!("Error: {error}\n").as_bytes()).await;
					}
					HubEvent::Executor(exec_event) => {
						// For now, we do not print those events.
						// This will be handled by the future Tui APP
					}
				}

				stdout.flush().await;
			}
		});

		Ok(())
	}
}

/// Lifecyle private functions
impl Tui {
	/// Execute the initial cli_args
	///
	/// Returns:
	///
	/// - The oneshot that will be executed after the executor_tx.send
	///
	/// Note: This function is designed to spawn it's on work and return the oneshot described above,
	///       so that it does not block the async caller.
	fn exec_cli_args(&self, cli_args: CliArgs) -> Result<oneshot::Receiver<()>> {
		let exec_cmd: ExecCommand = cli_args.cmd.into();
		let executor_tx = self.executor_tx();

		let (done_tx, done_rx) = oneshot::channel();
		tokio::spawn(async move {
			executor_tx.send(exec_cmd).await;
			done_tx.send(());
		});

		Ok(done_rx)
	}

	async fn wait_for_quit(&self, mut hub_rx: Receiver<HubEvent>) -> Result<()> {
		loop {
			#[allow(clippy::collapsible_match)] // will have more later.
			if let Ok(hub_event) = hub_rx.recv().await {
				if let HubEvent::Executor(ExecEvent::EndExec) = hub_event {
					break;
				}
			}
		}

		Ok(())
	}
}
