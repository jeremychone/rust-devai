//! Common utilities for path (local file path only) manipulation.
//! This is the beginning of the Unixy v.s. Windows os_normalization support

use std::path::Path;

/// Determine if the path is root based local path or not.
/// Simple `/` for unix and on Windows, do the `..:\` or `..:/` (sometime with rust) check
pub fn is_relative(path: impl AsRef<Path>) -> bool {
	let path = path.as_ref();
	!path.is_absolute()
}

pub fn path_last_components(path: impl AsRef<Path>, num: usize) -> String {
	let path = path.as_ref();
	let lasts: Vec<_> = path
		.components()
		.rev()
		.take(num)
		.collect::<Vec<_>>()
		.into_iter()
		.rev()
		.collect();
	lasts
		.into_iter()
		.map(|p| p.as_os_str().to_string_lossy().to_string())
		.collect::<Vec<_>>()
		.join("/")
}
