use crate::hub::{get_hub, Event};
use crate::Result;

#[derive(Debug, Default)]
pub struct Tui;

impl Tui {
	pub fn start_printer(&self) -> Result<()> {
		let mut rx = get_hub().subscriber();

		tokio::spawn(async move {
			while let Ok(event) = rx.recv().await {
				match event {
					Event::Message(msg) => {
						println!("{msg}");
					}
					Event::Error { error } => {
						println!("Error: {error}");
					}
				}
			}
		});

		Ok(())
	}
}
