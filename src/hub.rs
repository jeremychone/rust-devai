//! This module provides a global Hub for managing publish/subscribe events
//! using Tokio's broadcast channels. It allows multiple consumers to receive
//! real-time events concurrently in an efficient manner.
//!
//! # Design Pattern
//!
//! - The `Hub` struct encapsulates a multi-producer, multi-consumer (MPMC)
//!   broadcast channel, enabling multiple producers to publish events and
//!   multiple consumers to subscribe and receive events.
//!
//! - The struct holds a transmitter (`tx`) that is wrapped in an `Arc` for
//!   thread-safe sharing among multiple subscribers. A receiver instance (`_rx`)
//!   is stored internally to avoid dropping the connection.
//!
//! - A static instance of the `Hub` is created using `LazyLock`, ensuring
//!   that the global instance is initialized only once and is accessible
//!   throughout the application.
//!
//! - The `get_hub()` function provides a public way to access the global Hub
//!   instance.
//!
//! # Usage Example
//!
//! To use the Hub, you can subscribe to events in one task and publish
//! events in another:
//!
//! ```rust
//! use crate::hub::{get_hub, Event};
//!
//! #[tokio::main]
//! async fn main() {
//!     let hub = get_hub();
//!
//!     // Subscribe in one task
//!     let mut rx = hub.subscriber();
//!     tokio::spawn(async move {
//!         while let Ok(event) = rx.recv().await {
//!             match event {
//!                 Event::Message(msg) => {
//!                     println!("Received: {}", msg);
//!                 }
//!              }
//!         }
//!     });
//!
//!     // Publish an event
//!     hub.publish(Event::Message("Hello, world!".to_string())).await;
//! }
//! ```
//!
//! > NOTE: For this this hub has only one Event type. It might become a multi even type later.

use std::sync::{Arc, LazyLock};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
	Message(String), // An example event variant
}

// region:    --- Froms

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

// endregion: --- Froms

pub struct Hub {
	tx: Arc<broadcast::Sender<Event>>,
	// Store a subscriber receiver; we do not need to expose this publicly
	_rx: broadcast::Receiver<Event>,
}

impl Hub {
	// Create a new instance of Hub
	fn new() -> Self {
		// Create a new broadcast channel with a specified buffer size
		let (tx, _rx) = broadcast::channel(100); // Adjust buffer as needed
		Self { tx: Arc::new(tx), _rx }
	}

	// Publish an event to the global hub
	pub async fn publish(&self, event: impl Into<Event>) {
		// Send the event; handle the error appropriately in real applications
		let _ = self.tx.send(event.into());
	}

	// Return a new subscriber (receiver) instance
	pub fn subscriber(&self) -> broadcast::Receiver<Event> {
		self.tx.subscribe()
	}
}

// Using Lazy to instantiate a static Hub
static HUB: LazyLock<Hub> = LazyLock::new(Hub::new);

// Public function to get the global Hub instance
pub fn get_hub() -> &'static Hub {
	&HUB
}

// Example usage in an async context
#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_hub() {
		let hub = get_hub();

		// Subscribe in one task
		let mut rx = hub.subscriber();
		tokio::spawn(async move {
			while let Ok(event) = rx.recv().await {
				match event {
					Event::Message(msg) => {
						println!("Received: {}", msg);
					}
				}
			}
		});

		// Publish an event
		hub.publish(Event::Message("Hello, world!".to_string())).await;
	}
}
