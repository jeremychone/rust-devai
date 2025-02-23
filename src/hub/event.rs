use crate::Error;
use crate::exec::ExecEvent;
use derive_more::derive::From;
use std::sync::Arc;

/// HubEvent is sent by any part of the system that wants to share some information with the rest of the system.
/// For now, it is managed by the OutHub, which is a broadcast channel (to allow multiple listeners).
/// The types of events are:
/// - Message: Log message
/// - Error: Error occurred during some actions
///
/// Later, we will probably add the stage event:
/// - Stage(StageEvent): With StageEvent::BeforeAll, ...
/// - and others as they come along
///
/// Note: Also, more context will be added to those events for better reporting and such.
#[derive(Debug, Clone, From)]
pub enum HubEvent {
	Message(Arc<str>),
	Error {
		error: Arc<Error>,
	},
	#[from]
	Executor(ExecEvent),

	// -- Sent by the lua engine "print override"
	LuaPrint(Arc<str>),

	// -- Action event
	// for now, the watches send and event to the hub,
	// which will trigger the app to send it to the executor.
	DoExecRedo,

	// The quit events
	Quit,
}

// region:    --- Froms

// Implementing From trait for Event
impl From<String> for HubEvent {
	fn from(s: String) -> Self {
		HubEvent::Message(s.into())
	}
}

impl From<&str> for HubEvent {
	fn from(s: &str) -> Self {
		HubEvent::Message(s.into())
	}
}

impl From<&String> for HubEvent {
	fn from(s: &String) -> Self {
		HubEvent::Message(s.as_str().into())
	}
}

impl From<Error> for HubEvent {
	fn from(e: Error) -> Self {
		HubEvent::Error { error: Arc::new(e) }
	}
}

// endregion: --- Froms
