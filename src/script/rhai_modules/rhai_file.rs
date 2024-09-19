use crate::types::FileRecord;
use crate::Result;
use rhai::plugin::RhaiResult;
use rhai::{FuncRegistration, Module};
use simple_fs::ensure_file_dir;
use std::fs::write;
use std::path::Path;

pub fn rhai_module() -> Module {
	// Create a module for text functions
	let mut module = Module::new();

	FuncRegistration::new("load")
		.in_global_namespace()
		.set_into_module(&mut module, load);

	FuncRegistration::new("save")
		.in_global_namespace()
		.set_into_module(&mut module, save);

	module
}

// region:    --- Rhai Functions

fn load(file_path: &str) -> RhaiResult {
	let file_record = FileRecord::new(file_path);
	match file_record {
		Ok(file) => Ok(file.into()),
		Err(err) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
			format!("Failed to load file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

fn save(file_path: &str, content: &str) -> RhaiResult {
	fn file_save_inner(file_path: &str, content: &str) -> Result<()> {
		// let sfile = SFile::from_path(file_path)?;
		let path = Path::new(file_path);
		ensure_file_dir(path)?;
		write(path, content)?;
		println!("File saved: {}", file_path);
		Ok(())
	}

	match file_save_inner(file_path, content) {
		Ok(_) => Ok(().into()),
		Err(err) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
			format!("Failed to save file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

// endregion: --- Rhai Functions
