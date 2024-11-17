use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::{execute, terminal};
use flume::{Receiver, Sender};
use std::time::Duration;

/// The input reader (using crossterm)
/// This is to abstract the way we read user input for the devai app.
/// For now, simple key reader, and forward the crossterm KeyEvent in a flume rx
pub struct InReader {
	tx: Sender<KeyEvent>,
}

/// Constructor
impl InReader {
	pub fn new_and_rx() -> (Self, Receiver<KeyEvent>) {
		let (tx, rx) = flume::unbounded::<KeyEvent>();
		(Self { tx }, rx)
	}
}

impl InReader {
	pub fn start(&self) {
		let tx = self.tx.clone();

		// initialize
		terminal::enable_raw_mode().expect("Failed to enable crossterm raw mode");
		execute!(std::io::stdout(), Hide).expect("Failed to hide the cursor");

		tokio::spawn(async move {
			loop {
				/// FIXME: remove unwrap
				if event::poll(Duration::from_millis(10)).expect("Could not read/poll stdin") {
					if let Ok(Event::Key(key_event)) = event::read() {
						//
						if let Err(err) = tx.send_async(key_event).await {
							println!("InReader ERROR - sending key_event - {err}");
						};
					}
				}

				// Prevent CPU exhaustion
				tokio::task::yield_now().await;
			}
		});
	}

	pub fn close(&self) {
		terminal::disable_raw_mode().expect("Failed to disable crossterm raw mode");
		execute!(std::io::stdout(), Show).expect("Failed to show the cursor");
	}
}
