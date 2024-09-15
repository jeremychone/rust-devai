use crate::{Error, Result};
use rhai::{Dynamic, Engine};
use simple_fs::{read_to_string, SFile};
use std::sync::{Arc, LazyLock};

type RhaiResult = core::result::Result<Dynamic, Box<rhai::EvalAltResult>>;

// Create a lazy-initialized engine with registered functions
static ENGINE: LazyLock<Arc<Engine>> = LazyLock::new(|| {
	let mut engine = Engine::new();

	// Register custom functions
	engine.register_fn("load_file", load_file_rhai);

	engine.into()
});

pub(super) fn rhai_engine() -> Result<Arc<Engine>> {
	Ok(ENGINE.clone())
}

// region:    --- Scripts

pub struct File {
	pub name: String,
	pub path: String,
	pub content: String,
}

// Implement conversion from File to Dynamic
impl From<File> for Dynamic {
	fn from(file: File) -> Dynamic {
		let mut map = rhai::Map::new();
		map.insert("name".into(), file.name.into());
		map.insert("path".into(), file.path.into());
		map.insert("content".into(), file.content.into());
		Dynamic::from_map(map)
	}
}

// Implement conversion from Dynamic to File
impl TryFrom<Dynamic> for File {
	type Error = crate::Error;

	fn try_from(value: Dynamic) -> Result<Self> {
		let map = value.cast::<rhai::Map>();
		Ok(File {
			name: map.get("name").ok_or("missing name property")?.clone().cast::<String>(),
			path: map.get("path").ok_or("missing path property")?.clone().cast::<String>(),
			content: map.get("content").ok_or("missing content property")?.clone().cast::<String>(),
		})
	}
}

fn load_file_rhai(file_path: &str) -> RhaiResult {
	match load_file(file_path) {
		Ok(file) => Ok(file.into()),
		Err(err) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
			format!("Failed to load file: {}", err).into(),
			rhai::Position::NONE,
		))),
	}
}

fn load_file(file_path: &str) -> Result<File> {
	let sfile = SFile::from_path(file_path)?;
	let content = read_to_string(&sfile)?;
	Ok(File {
		name: sfile.file_name().to_string(),
		path: sfile.to_string(),
		content,
	})
}

// endregion: --- Scripts
