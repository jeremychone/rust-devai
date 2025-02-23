//! The executor command
//! Note: For now, the content of the variant of the ExecCommand often contain the CliArgs,
//!       but this will eventual change to have it's own

use crate::cli::{InitArgs, ListArgs, NewArgs, RunArgs};

/// This is the Executor Command that needs to be performed
/// NOTE: This is not the `ExecStateEvent` which is sent to the hub.
#[derive(Debug)]
pub enum ExecCommand {
	Init(InitArgs),
	InitBase,
	RunCommandAgent(RunArgs),
	#[allow(unused)]
	NewCommandAgent(NewArgs),
	List(ListArgs),
	Redo,
	OpenAgent,
}
