use crate::hub::{get_hub, Event};
use crate::Result;

pub struct Tui;

impl Tui {
	pub fn start_printer() -> Result<()> {
		let mut rx = get_hub().subscriber();

		tokio::spawn(async move {
			while let Ok(event) = rx.recv().await {
				match event {
					Event::Message(msg) => {
						println!("Received:\n{}", msg);
					}
				}
			}
		});

		Ok(())
	}
}
