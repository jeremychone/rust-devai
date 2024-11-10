use super::rhai_modules::{rhai_devai, rhai_file, rhai_git, rhai_md, rhai_text};
use crate::run::RuntimeContext;
use crate::script::rhai_script::rhai_modules::{rhai_html, rhai_path, rhai_rust, rhai_web};
use crate::Result;
use rhai::Engine;

// use std::sync::{Arc, LazyLock};
// // Create a lazy-initialized engine with registered functions
// static ENGINE: LazyLock<Arc<Engine>> = LazyLock::new(|| {
// 	let mut engine = Engine::new();

// 	engine.register_static_module("file", rhai_file::rhai_module().into());
// 	engine.register_static_module("path", rhai_path::rhai_module().into());
// 	engine.register_static_module("devai", rhai_devai::rhai_module().into());
// 	engine.register_static_module("web", rhai_web::rhai_module().into());
// 	engine.register_static_module("git", rhai_git::rhai_module().into());
// 	engine.register_static_module("text", rhai_text::rhai_module().into());
// 	engine.register_static_module("md", rhai_md::rhai_module().into());
// 	engine.register_static_module("html", rhai_html::rhai_module().into());
// 	engine.register_static_module("rust", rhai_rust::rhai_module().into());

// 	engine.into()
// });

// pub(super) fn rhai_engine() -> Result<Arc<Engine>> {
// 	Ok(ENGINE.clone())
// }

pub fn new_rhai_engine(runtime_context: RuntimeContext) -> Result<Engine> {
	let mut engine = Engine::new();

	engine.register_static_module("file", rhai_file::rhai_module(&runtime_context).into());
	engine.register_static_module("path", rhai_path::rhai_module(&runtime_context).into());
	engine.register_static_module("devai", rhai_devai::rhai_module(&runtime_context).into());
	engine.register_static_module("web", rhai_web::rhai_module().into());
	engine.register_static_module("git", rhai_git::rhai_module().into());
	engine.register_static_module("text", rhai_text::rhai_module().into());
	engine.register_static_module("md", rhai_md::rhai_module().into());
	engine.register_static_module("html", rhai_html::rhai_module().into());
	engine.register_static_module("rust", rhai_rust::rhai_module().into());

	Ok(engine)
}
