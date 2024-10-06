use crate::Error;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Event {
	Message(String),
	Error { error: Arc<Error> },
}

// region:    --- Froms

// Implementing From trait for Event
impl From<String> for Event {
	fn from(s: String) -> Self {
		Event::Message(s)
	}
}

impl From<&str> for Event {
	fn from(s: &str) -> Self {
		Event::Message(s.to_string())
	}
}

impl From<&String> for Event {
	fn from(s: &String) -> Self {
		Event::Message(s.clone())
	}
}

impl From<Error> for Event {
	fn from(e: Error) -> Self {
		Event::Error { error: Arc::new(e) }
	}
}

// endregion: --- Froms
