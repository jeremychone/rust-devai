use crate::hub::get_hub;
use crate::run::{get_genai_client, DirContext};
use crate::script::new_rhai_engine;
use crate::{Error, Result};
use flume::{Receiver, Sender};
use genai::Client;
use rhai::Engine;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Clone)]
pub struct Runtime {
	context: RuntimeContext,
	rhai_engine: Arc<Engine>,
	stop_signal: Arc<watch::Sender<()>>, // Signal to stop the task
}

/// Constructors
impl Runtime {
	pub fn new(dir_context: DirContext) -> Result<Self> {
		// Note: Make the type explicit for clarity
		let (tx, rx): (Sender<Sender<Runtime>>, Receiver<Sender<Runtime>>) = flume::unbounded();
		let client = get_genai_client()?;

		let context = RuntimeContext::new(dir_context, client, tx);

		let rhai_engine = new_rhai_engine(context.clone())?;
		let rhai_engine = Arc::new(rhai_engine);

		let rx = Arc::new(rx);
		let (stop_signal, stop_receiver) = watch::channel(()); // Stop signal
		let stop_signal = Arc::new(stop_signal);
		let runtime = Self {
			context,
			rhai_engine,
			stop_signal,
		};

		// -- Process to listen for Runtime requests
		// NOTE: This is a workaround since we need the Runtime to have a rhai_engine,
		//       but we need the rhai_engine to be built with the RuntimeContext.
		//       For devai::run, the function will need to get the engine back.
		let runtime_for_rx = runtime.clone();
		let stop_receiver = stop_receiver.clone();
		tokio::spawn(async move {
			let mut stop_receiver = stop_receiver; // Keep the mutable receiver
			loop {
				tokio::select! {
					_ = stop_receiver.changed() => {
						// Stop signal received, exit loop
						break;
					}
					recv_result = rx.recv_async() => {
						match recv_result {
							Ok(one_tx) => {
								// Send back a clone of the runtime
								if let Err(send_err) = one_tx.send(runtime_for_rx.clone()) {
									get_hub().publish_sync(Error::cc("Runtime send error", send_err));
								}
							}
							Err(recv_err) => {
								get_hub().publish_sync(Error::cc("Runtime rx error", recv_err));
								break; // Exit loop on receiver error
							}
						}
					}
				}
			}
		});

		Ok(runtime)
	}

	#[cfg(test)]
	pub fn new_for_test() -> Result<Self> {
		let dir_context = DirContext::from_parent_dir("./")?;
		Self::new(dir_context)
	}
}

/// We implement Drop to ensure we send an event to stop the process/task
/// that started in the new instance
impl Drop for Runtime {
	fn drop(&mut self) {
		// Notify the spawned task to stop
		let _ = self.stop_signal.send(());
	}
}

/// Getters
impl Runtime {
	pub fn context(&self) -> RuntimeContext {
		self.context.clone()
	}

	pub fn rhai_engine(&self) -> &Arc<Engine> {
		&self.rhai_engine
	}

	pub fn genai_client(&self) -> &Client {
		self.context.genai_client()
	}

	pub fn dir_context(&self) -> &DirContext {
		self.context.dir_context()
	}
}

// region:    --- RuntimeContext

#[derive(Clone)]
pub struct RuntimeContext {
	inner: Arc<RuntimeContextInner>,
	tx: Sender<Sender<Runtime>>,
}

/// Constructors
impl RuntimeContext {
	pub fn new(dir_context: DirContext, genai_client: Client, tx: Sender<Sender<Runtime>>) -> Self {
		Self {
			inner: Arc::new(RuntimeContextInner {
				dir_context,
				genai_client,
			}),
			tx,
		}
	}
}

/// Getters
impl RuntimeContext {
	pub fn dir_context(&self) -> &DirContext {
		&self.inner.dir_context
	}

	pub fn genai_client(&self) -> &Client {
		&self.inner.genai_client
	}

	pub fn get_runtime(&self) -> Result<Runtime> {
		let (one_tx, one_rx) = flume::bounded(1);
		self.tx
			.send(one_tx)
			.map_err(|err| Error::cc("RuntimeContext::get_runtime ", err))?;
		let runtime = one_rx.recv().map_err(|err| Error::cc("RuntimeContext::get_runtime ", err))?;

		Ok(runtime)
	}
}

struct RuntimeContextInner {
	dir_context: DirContext,
	genai_client: Client,
}

// endregion: --- RuntimeContext
