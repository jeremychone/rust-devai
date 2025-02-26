use crate::dir_context::DirContext;
use genai::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct RuntimeContext {
	inner: Arc<RuntimeContextInner>,
}

/// Constructors
impl RuntimeContext {
	pub fn new(dir_context: DirContext, genai_client: Client) -> Self {
		Self {
			inner: Arc::new(RuntimeContextInner {
				dir_context,
				genai_client,
			}),
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
}

struct RuntimeContextInner {
	dir_context: DirContext,
	genai_client: Client,
}
