// region:    --- Modules

mod in_reader;
mod tui_elem;

mod tui_app;

pub use tui_app::*;
// NOTE: for now, we expose those tui_elem, but later, all should go through tui
pub use tui_elem::*;

// endregion: --- Modules
