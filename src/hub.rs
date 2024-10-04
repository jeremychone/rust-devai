use std::sync::{Arc, LazyLock};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
	Message(String), // An example event variant
}

pub struct Hub {
	tx: Arc<broadcast::Sender<Event>>,
	// Store a subscriber receiver; we do not need to expose this publicly
	_rx: broadcast::Receiver<Event>,
}

impl Hub {
	// Create a new instance of Hub
	pub fn new() -> Self {
		// Create a new broadcast channel with a specified buffer size
		let (tx, _rx) = broadcast::channel(100); // Adjust buffer as needed
		Self { tx: Arc::new(tx), _rx }
	}

	// Publish an event to the global hub
	pub async fn publish(&self, event: Event) {
		// Send the event; handle the error appropriately in real applications
		let _ = self.tx.send(event);
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
