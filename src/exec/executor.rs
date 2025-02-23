//! The command executor.
//! Will create it's own queue and listen to ExecCommand events.

use crate::agent::Agent;
use crate::exec::exec_command::ExecCommand;
use crate::exec::support::open_vscode;
use crate::exec::{ExecEvent, RunRedoCtx, exec_list, exec_new, exec_run, exec_run_redo};
use crate::hub::get_hub;
use crate::init::{init_base, init_wks};
use crate::{Error, Result};
use derive_more::derive::From;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

// region:    --- RedoCtx

#[derive(From)]
enum RedoCtx {
	RunRedoCtx(Arc<RunRedoCtx>),
}

impl From<RunRedoCtx> for RedoCtx {
	fn from(run_redo_ctx: RunRedoCtx) -> Self {
		RedoCtx::RunRedoCtx(run_redo_ctx.into())
	}
}

impl RedoCtx {
	pub fn get_agent(&self) -> Option<&Agent> {
		match self {
			RedoCtx::RunRedoCtx(redo_ctx) => Some(redo_ctx.agent()),
		}
	}
}

// endregion: --- RedoCtx

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

	/// Return the latest agent file_path that was executed
	fn get_agent_file_path(&self) -> Option<&str> {
		Some(self.current_redo_ctx.as_ref()?.get_agent()?.file_path())
	}
}

/// Runner
impl Executor {
	pub async fn start(&mut self) -> Result<()> {
		let hub = get_hub();

		loop {
			let Some(cmd) = self.command_rx.recv().await else {
				println!("!!!! Aipack Executor: Channel closed");
				break;
			};

			hub.publish(ExecEvent::StartExec).await;

			match cmd {
				ExecCommand::Init(init_args) => {
					init_wks(init_args.path.as_deref(), true).await?;
					init_base(false).await?;
				}
				ExecCommand::InitBase => {
					init_base(true).await?;
				}
				ExecCommand::NewCommandAgent(new_args) => {
					exec_new(new_args, init_wks(None, false).await?).await?;
				}
				ExecCommand::List(list_args) => exec_list(init_wks(None, false).await?, list_args).await?,

				ExecCommand::RunCommandAgent(run_args) => {
					hub.publish(ExecEvent::RunStart).await;
					let redo = exec_run(run_args, init_wks(None, false).await?).await?;
					self.current_redo_ctx = Some(redo.into());
					hub.publish(ExecEvent::RunEnd).await;
				}

				ExecCommand::Redo => {
					let Some(redo_ctx) = self.current_redo_ctx.as_ref() else {
						hub.publish(Error::custom("No redo available to be performed")).await;
						continue;
					};

					hub.publish(ExecEvent::RunStart).await;
					match redo_ctx {
						RedoCtx::RunRedoCtx(redo_ctx) => {
							// if sucessul, we recapture the redo_ctx to have the latest agent.
							if let Some(redo_ctx) = exec_run_redo(redo_ctx).await {
								self.current_redo_ctx = Some(redo_ctx.into())
							}
						}
					}
					hub.publish(ExecEvent::RunEnd).await;
				}

				ExecCommand::OpenAgent => {
					//
					if let Some(agent_file_path) = self.get_agent_file_path() {
						open_vscode(agent_file_path).await
					}
				}
			}

			hub.publish(ExecEvent::EndExec).await;
		}

		Ok(())
	}
}
