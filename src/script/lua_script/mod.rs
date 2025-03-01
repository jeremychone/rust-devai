// region:    --- Modules

mod helpers;
mod lua_engine;
mod lua_value_ext;
mod utils_aipack;
mod utils_cmd;
mod utils_code;
mod utils_file;
mod utils_git;
mod utils_hbs;
mod utils_html;
mod utils_json;
mod utils_lua;
mod utils_md;
mod utils_path;
mod utils_rust;
mod utils_text;
mod utils_web;

pub use lua_engine::*;
pub use lua_value_ext::*;

#[cfg(test)]
pub use helpers::*;

// endregion: --- Modules

const DEFAULT_MARKERS: &(&str, &str) = &("<<START>>", "<<END>>");
