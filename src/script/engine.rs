use crate::script::rhai_modules::{rhai_file, rhai_git, rhai_md, rhai_text};
use crate::Result;
use rhai::Engine;
use std::sync::{Arc, LazyLock};

// Create a lazy-initialized engine with registered functions
static ENGINE: LazyLock<Arc<Engine>> = LazyLock::new(|| {
	let mut engine = Engine::new();

	engine.register_static_module("text", rhai_text::rhai_module().into());
	engine.register_static_module("file", rhai_file::rhai_module().into());
	engine.register_static_module("md", rhai_md::rhai_module().into());
	engine.register_static_module("git", rhai_git::rhai_module().into());

	engine.into()
});

pub(super) fn rhai_engine() -> Result<Arc<Engine>> {
	Ok(ENGINE.clone())
}
