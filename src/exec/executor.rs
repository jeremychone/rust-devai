//! The command executor.
//! Will create it's own queue and listen to ExecCommand events.

use crate::exec::exec_command::ExecCommand;
use crate::exec::{exec_list, exec_new, exec_new_solo, exec_run, exec_solo, ExecEvent};
use crate::hub::get_hub;
use crate::init::init_devai_files;
use crate::Result;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct Executor {
	rx: Receiver<ExecCommand>,
	_tx: Sender<ExecCommand>,
}

/// Contructor
impl Executor {
	pub fn new() -> Self {
		let (_tx, rx) = channel(100);
		Executor { rx, _tx }
	}
}

/// Getter
impl Executor {
	pub fn tx(&self) -> Sender<ExecCommand> {
		self._tx.clone()
	}
}

/// Runner
impl Executor {
	pub async fn start(&mut self) -> Result<()> {
		let out_hub = get_hub();

		loop {
			let Some(cmd) = self.rx.recv().await else {
				println!("!!!! Executor: Channel closed");
				break;
			};

			out_hub.publish(ExecEvent::StartExec).await;

			match cmd {
				ExecCommand::Init(init_args) => {
					init_devai_files(init_args.path.as_deref(), true).await?;
				}
				ExecCommand::RunCommandAgent(run_args) => {
					exec_run(run_args, init_devai_files(None, false).await?).await?;
				}
				ExecCommand::RunSoloAgent(solo_args) => {
					exec_solo(solo_args, init_devai_files(None, false).await?).await?;
				}
				ExecCommand::NewCommandAgent(new_args) => {
					exec_new(new_args, init_devai_files(None, false).await?).await?;
				}
				ExecCommand::NewSoloAgent(new_solo_args) => {
					exec_new_solo(new_solo_args, init_devai_files(None, false).await?).await?;
				}
				ExecCommand::List => exec_list(init_devai_files(None, false).await?).await?,
			}

			out_hub.publish(ExecEvent::EndExec).await;
		}

		Ok(())
	}
}
