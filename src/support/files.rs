use crate::{Error, Result};
use simple_fs::SPath;
use std::env;

pub fn current_dir() -> Result<SPath> {
	let dir = env::current_dir().map_err(|err| Error::cc("Current dir error", err))?;
	let dir = SPath::new(dir)?;
	Ok(dir)
}
