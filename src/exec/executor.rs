//! The command executor.
//! Will create it's own queue and listen to ExecCommand events.

use crate::exec::exec_command::ExecCommand;
use crate::exec::{
	exec_list, exec_new, exec_new_solo, exec_run, exec_run_redo, exec_solo, ExecEvent, RunRedoCtx, SoloRedoCtx,
};
use crate::hub::get_hub;
use crate::init::init_devai_files;
use crate::{Error, Result};
use derive_more::derive::From;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(From)]
enum RedoCtx {
	#[from]
	RunRedoCtx(Arc<RunRedoCtx>),
	#[from]
	SoloRedoCtx(Arc<SoloRedoCtx>),
}

pub struct Executor {
	/// The receiver that this executor will itreate on "start"
	command_rx: Receiver<ExecCommand>,
	/// Sender that gets cloned for parts that want to send events
	command_tx: Sender<ExecCommand>,

	current_redo_ctx: Option<RedoCtx>,
}

/// Contructor
impl Executor {
	pub fn new() -> Self {
		let (_tx, rx) = channel(100);
		Executor {
			command_rx: rx,
			command_tx: _tx,
			current_redo_ctx: None,
		}
	}
}

/// Getter
impl Executor {
	pub fn command_tx(&self) -> Sender<ExecCommand> {
		self.command_tx.clone()
	}
}

/// Runner
impl Executor {
	pub async fn start(&mut self) -> Result<()> {
		let hub = get_hub();

		loop {
			let Some(cmd) = self.command_rx.recv().await else {
				println!("!!!! Devai Executor: Channel closed");
				break;
			};

			hub.publish(ExecEvent::StartExec).await;

			match cmd {
				ExecCommand::Init(init_args) => {
					init_devai_files(init_args.path.as_deref(), true).await?;
				}
				ExecCommand::RunCommandAgent(run_args) => {
					let redo = exec_run(run_args, init_devai_files(None, false).await?).await?;
					self.current_redo_ctx = Some(redo.into());
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

				ExecCommand::Redo => {
					let Some(redo_ctx) = self.current_redo_ctx.as_ref() else {
						hub.publish(Error::custom("No redo available to be performed")).await;
						continue;
					};
					match redo_ctx {
						RedoCtx::RunRedoCtx(redo_ctx) => exec_run_redo(redo_ctx).await,
						RedoCtx::SoloRedoCtx(arc) => todo!(),
					}
				}
			}

			hub.publish(ExecEvent::EndExec).await;
		}

		Ok(())
	}
}
