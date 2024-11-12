// region:    --- Modules

pub mod rhai_devai;
pub mod rhai_file;
pub mod rhai_file_md;
pub mod rhai_git;
pub mod rhai_html;
pub mod rhai_md;
pub mod rhai_path;
pub mod rhai_rust;
pub mod rhai_text;
pub mod rhai_web;

// endregion: --- Modules

// Here we put it here because of the devai makers, otherwise, we cannot use devai on rhai_text
const DEFAULT_MARKERS: &(&str, &str) = &("<<START>>", "<<END>>");
