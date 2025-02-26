use crate::Result;
use crate::dir_context::DirContext;
use crate::run::{RuntimeContext, get_genai_client};
use crate::script::LuaEngine;
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
		use crate::_test_support::{SANDBOX_01_BASE_AIPACK_DIR, SANDBOX_01_WKS_DIR};
		use crate::dir_context::AipackPaths;
		use simple_fs::SPath;
		use std::path::Path;

		let current_dir = Path::new(SANDBOX_01_WKS_DIR).canonicalize()?;
		let current_dir = SPath::new(current_dir)?;

		let wks_aipack_dir = current_dir.join_str(".aipack");

		let base_aipack_dir = Path::new(SANDBOX_01_BASE_AIPACK_DIR).canonicalize()?;
		let base_aipack_dir = SPath::new(base_aipack_dir)?;

		let aipack_paths = AipackPaths::from_aipack_base_and_wks_dirs(base_aipack_dir, wks_aipack_dir)?;

		let dir_context = DirContext::from_current_and_aipack_paths(current_dir, aipack_paths)?;

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
