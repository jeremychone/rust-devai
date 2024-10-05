// src/hub/event.rs

#[derive(Debug, Clone)]
pub enum Event {
    Message(String),
}

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
