use crate::run::{DirContext, Runtime};
use crate::{Error, Result};
use flume::Sender;
use genai::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct RuntimeContext {
	inner: Arc<RuntimeContextInner>,
	runtime_getter_tx: Sender<Sender<Runtime>>,
}

/// Constructors
impl RuntimeContext {
	pub fn new(dir_context: DirContext, genai_client: Client, tx: Sender<Sender<Runtime>>) -> Self {
		Self {
			inner: Arc::new(RuntimeContextInner {
				dir_context,
				genai_client,
			}),
			runtime_getter_tx: tx,
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

	/// Get the runtime which contains this RuntimeContext.
	/// Note: This is for the Rhai `devai::run` for example which needs to get back the full runtime to call the `run_command...`
	pub fn get_runtime(&self) -> Result<Runtime> {
		let (one_tx, one_rx) = flume::bounded(1);
		self.runtime_getter_tx
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
