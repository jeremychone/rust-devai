//! Base module for the script engine.
//!
//! > NOTE 1: Currently, devai only supports one scripting engine, but the strategy is to eventually support at least one
//! >         more scripting engine (either Python, JavaScript, or LUA).
//! >         Python is the likely candidate, provided RustPython does not have any major issues.
//!
//! > NOTE 2: For now, we will flatten the rhai_script

// region:    --- Modules

mod rhai_script;

pub use rhai_script::*;

// endregion: --- Modules
