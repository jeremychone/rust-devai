//! The executor command
//! Note: For now, the content of the variant of the ExecCommand often contain the CliArgs,
//!       but this will eventual change to have it's own

use crate::cli::{InitArgs, NewArgs, NewSoloArgs, RunArgs, SoloArgs};

//
#[derive(Debug)]
pub enum ExecCommand {
	Init(InitArgs),
	RunCommandAgent(RunArgs),
	RunSoloAgent(SoloArgs),
	NewCommandAgent(NewArgs),
	NewSoloAgent(NewSoloArgs),
	List,
}
