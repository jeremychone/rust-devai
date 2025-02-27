// region:    --- Modules

mod as_strs_ext;
mod common;
mod cow_lines;
mod str_ext;

pub use as_strs_ext::*;
pub use common::*;
pub use cow_lines::*;
pub use str_ext::*;

pub mod code;
pub mod cred;
pub mod files;
pub mod hbs;
pub mod html;
pub mod jsons;
pub mod md;
pub mod paths;
pub mod text;
pub mod tomls;
pub mod zip;

// endregion: --- Modules

/// Generic wrapper for a NewType Pattern
pub struct W<T>(pub T);
