// region:    --- Module

mod assets;

mod init_base;
mod init_wks;

pub use init_base::*;
pub use init_wks::*;
pub use assets::{extract_template_pack_toml_zfile, extract_template_zfile};

// endregion: --- Module
