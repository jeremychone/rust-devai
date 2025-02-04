// region:    --- Modules

mod as_strs_ext;
mod cow_lines;

pub use as_strs_ext::*;
pub use cow_lines::*;

pub mod code;
pub mod cred;
pub mod files;
pub mod hbs;
pub mod html;
pub mod jsons;
pub mod md;
pub mod text;
pub mod tomls;

// endregion: --- Modules

/// Generic wrapper for a NewType Pattern
pub struct W<T>(pub T);
