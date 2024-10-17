// region:    --- Modules

mod base;
mod hub_capture;

pub use base::*;
pub use hub_capture::*;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

// endregion: --- Modules
