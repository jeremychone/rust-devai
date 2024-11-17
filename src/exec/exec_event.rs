//! The executor event

use derive_more::derive::Display;

/// This is the status event sent by the executor to the Hub.
///
/// NOTE: This is not sent to the executor.command_tx is they are not commands,
///       but status events.
#[derive(Debug, Clone, Display)]
pub enum ExecEvent {
	/// Start an exec command like run, solo, init, ...
	StartExec,

	/// The end of an exec command
	/// Note: When watch mode is on, the EndExec will be sent after the watch mode is started,
	///       but not when it finished (because it won't finished by definition)
	///       So, EndExec always get triggered for each ExecCommand
	EndExec,

	/// This is send at the end of a watch redo.
	EndWatchRedo,
}
