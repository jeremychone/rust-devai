// src/hub/hub_base.rs

use crate::hub::event::HubEvent;
use std::sync::{Arc, LazyLock};
use tokio::sync::broadcast;

/// Hub for receiving and broadcasting all OutEvent to the systems.
/// Those events are Log Message, Error, and Stage(StagEvent) to capture each progress steps
pub struct OutHub {
	tx: Arc<broadcast::Sender<HubEvent>>,
	_rx: broadcast::Receiver<HubEvent>,
}

impl OutHub {
	pub fn new() -> Self {
		let (tx, _rx) = broadcast::channel(10);
		Self { tx: Arc::new(tx), _rx }
	}

	pub async fn publish(&self, event: impl Into<HubEvent>) {
		let event = event.into();

		match self.tx.send(event) {
			Ok(_) => (),
			Err(err) => println!("DEVAI INTERNAL ERROR - failed to send event to hub - {err}"),
		}
	}

	pub fn publish_sync(&self, event: impl Into<HubEvent>) {
		tokio::task::block_in_place(|| {
			let event = event.into();
			let rt = tokio::runtime::Handle::try_current();
			match rt {
				Ok(rt) => rt.block_on(async { self.publish(event).await }),

				// NOTE: Here per design, we do not return error or break, as it is just for logging
				Err(err) => println!("DEVAI INTERNAL ERROR - no current tokio handle - {err}"),
			}
		});
	}

	pub fn subscriber(&self) -> broadcast::Receiver<HubEvent> {
		self.tx.subscribe()
	}
}

static OUT_HUB: LazyLock<OutHub> = LazyLock::new(OutHub::new);

pub fn get_hub() -> &'static OutHub {
	&OUT_HUB
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
				#[allow(clippy::single_match)]
				match event {
					HubEvent::Message(msg) => {
						println!("Received Message: {}", msg);
					}
					_ => (),
				}
			}
		});

		// Testing async publish
		hub.publish(HubEvent::Message("Hello, world!".into())).await;

		// NOTE: Call below will fail in test because require multi-thread
		// hub.publish_sync(Event::Message("Hello from sync!".to_string()));
	}
}
