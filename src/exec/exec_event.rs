//! The executor event

use crate::hub::HubEvent;
use derive_more::derive::Display;

#[derive(Debug, Clone, Display)]
pub enum ExecEvent {
	StartExec,
	EndExec,
}
