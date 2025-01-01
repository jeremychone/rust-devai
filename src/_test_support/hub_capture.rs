use crate::_test_support::Result;
use crate::hub::{get_hub, HubEvent};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

#[allow(unused)]
pub struct HubCapture {
	content: Arc<Mutex<String>>,
	stop_signal: Option<oneshot::Sender<()>>,
}

#[allow(unused)]
impl HubCapture {
	pub fn new_and_start() -> Self {
		let (stop_tx, stop_rx) = oneshot::channel();
		let content = Arc::new(Mutex::new(String::new()));
		let content_clone = Arc::clone(&content);
		let mut rx = get_hub().subscriber();

		// Spawn a background task to handle the events and stop signal
		tokio::spawn(async move {
			tokio::select! {
				_ = stop_rx => {
					// Stop signal received, exit the loop
				}
				_ = async {
					while let Ok(event) = rx.recv().await {
						match event {
							HubEvent::Message(msg) => {
								let mut content = content_clone.lock().await;
								content.push_str(&msg);
								content.push('\n');
							}
							HubEvent::Error { error } => {
								let mut content = content_clone.lock().await;
								content.push_str(&format!("Error: {error}\n"));
							},

							HubEvent::LuaPrint(text) => {
								let mut content = content_clone.lock().await;
								content.push_str(&format!("LuaPrint: {text}\n"));
							}
							HubEvent::Executor(exec_event)=> {
								let mut content = content_clone.lock().await;
								content.push_str(&format!("Exec: {exec_event} \n"));
							},
							HubEvent::DoExecRedo => {
								let mut content = content_clone.lock().await;
								content.push_str("DoExecRedo\n");
							}
							HubEvent::Quit => {
								let mut content = content_clone.lock().await;
								content.push_str("Quit\n");
							},

						}
					}
				} => {
					// The event receiver loop completes
				}
			}
		});

		Self {
			content,
			stop_signal: Some(stop_tx),
		}
	}

	pub async fn into_content(mut self) -> Result<String> {
		// Send stop signal to stop the background polling
		if let Some(stop_tx) = self.stop_signal.take() {
			let _ = stop_tx.send(());
		}

		// Lock the content and retrieve its value
		let mut content = self.content.lock().await;
		let new_string = std::mem::take(&mut *content);

		Ok(new_string)
	}
}
