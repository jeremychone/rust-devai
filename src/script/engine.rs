use crate::support::md;
use crate::types::FileRecord;
use crate::{Error, Result};
use rhai::{Dynamic, Engine};
use serde::Serialize;
use simple_fs::{ensure_file_dir, read_to_string, SFile};
use std::fs::write;
use std::path::Path;
use std::sync::{Arc, LazyLock};

type RhaiResult = core::result::Result<Dynamic, Box<rhai::EvalAltResult>>;

// Create a lazy-initialized engine with registered functions
static ENGINE: LazyLock<Arc<Engine>> = LazyLock::new(|| {
	let mut engine = Engine::new();

	// Register custom functions
	engine.register_fn("file_load", file_load_rhai);
	engine.register_fn("file_save", file_save_rhai);
	engine.register_fn("md_extract_first_rust", md_extract_first_rust_rhai);

	engine.into()
});

pub(super) fn rhai_engine() -> Result<Arc<Engine>> {
	Ok(ENGINE.clone())
}

// region:    --- md_extract_first_rust

fn md_extract_first_rust_rhai(content: &str) -> RhaiResult {
	let rust_content = md::md_extract_first_rust_block(content);

	match rust_content {
		Some(rust_content) => Ok(rust_content.into()),
		None => Ok(Dynamic::FALSE),
	}
}

// endregion: --- md_extract_first_rust

// region:    --- file_load

fn file_load_rhai(file_path: &str) -> RhaiResult {
	let file_record = FileRecord::new(file_path);
	match file_record {
		Ok(file) => Ok(file.into()),
		Err(err) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
			format!("Failed to load file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

// endregion: --- file_load

// region:    --- file_save

fn file_save_rhai(file_path: &str, content: &str) -> RhaiResult {
	match file_save(file_path, content) {
		Ok(_) => Ok(().into()),
		Err(err) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
			format!("Failed to load file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

fn file_save(file_path: &str, content: &str) -> Result<()> {
	// let sfile = SFile::from_path(file_path)?;
	let path = Path::new(file_path);
	ensure_file_dir(path)?;
	write(path, content);
	println!("File saved: {}", file_path);
	Ok(())
}

// endregion: --- file_load
