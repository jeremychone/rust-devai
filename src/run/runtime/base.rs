use crate::run::{get_genai_client, DirContext, RuntimeContext};
use crate::script::LuaEngine;
use crate::Result;
use genai::Client;

#[derive(Clone)]
pub struct Runtime {
	context: RuntimeContext,
}

/// Constructors
impl Runtime {
	pub fn new(dir_context: DirContext) -> Result<Self> {
		// Note: Make the type explicit for clarity
		let client = get_genai_client()?;

		let context = RuntimeContext::new(dir_context, client);

		let runtime = Self { context };

		Ok(runtime)
	}

	#[cfg(test)]
	pub fn new_test_runtime_sandbox_01() -> Result<Self> {
		use crate::_test_support::SANDBOX_01_DIR;
		use simple_fs::SPath;

		let dir_context =
			DirContext::from_parent_dir_and_current_dir_for_test(SANDBOX_01_DIR, SPath::new(SANDBOX_01_DIR)?)?;
		Self::new(dir_context)
	}
}

/// lua engine
/// NOTE: For now, we do not keep any Lau engine in the Runtime, but just create new ones.
///       Later, we might have an optmized reuse strategy of lua engines (but need to be cautious as not multi-threaded)
impl Runtime {
	pub fn new_lua_engine(&self) -> Result<LuaEngine> {
		LuaEngine::new(self.context.clone())
	}
}

/// Getters
impl Runtime {
	#[allow(unused)]
	pub fn context(&self) -> RuntimeContext {
		self.context.clone()
	}

	pub fn genai_client(&self) -> &Client {
		self.context.genai_client()
	}

	pub fn dir_context(&self) -> &DirContext {
		self.context.dir_context()
	}
}
