// src/hub/hub_base.rs

use crate::hub::event::Event;
use std::sync::{Arc, LazyLock};
use tokio::sync::broadcast;

pub struct Hub {
	tx: Arc<broadcast::Sender<Event>>,
	_rx: broadcast::Receiver<Event>,
}

impl Hub {
	pub fn new() -> Self {
		let (tx, _rx) = broadcast::channel(100);
		Self { tx: Arc::new(tx), _rx }
	}

	pub async fn publish(&self, event: impl Into<Event>) {
		let _ = self.tx.send(event.into());
	}

	pub fn publish_sync(&self, event: impl Into<Event>) {
		tokio::task::block_in_place(|| {
			let event = event.into();
			let rt = tokio::runtime::Handle::try_current();
			match rt {
				Ok(rt) => rt.block_on(async { self.publish(event).await }),
				Err(_) => println!("DEVAI INTERNAL ERROR - no current tokio handle"),
			}
		});
	}

	pub fn subscriber(&self) -> broadcast::Receiver<Event> {
		self.tx.subscribe()
	}
}

static HUB: LazyLock<Hub> = LazyLock::new(Hub::new);

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

		// Testing async publish
		hub.publish(Event::Message("Hello, world!".to_string())).await;

		// NOTE: Call below will fail in test because require multi-thread
		// hub.publish_sync(Event::Message("Hello from sync!".to_string()));
	}
}
