use crate::exec::SoloConfig;
use simple_fs::SPath;

#[derive(Debug, Clone)]
pub struct AiSoloConfig {
	verbose: bool,
	solo_path: SPath,
	target_path: SPath,
}

impl AiSoloConfig {
	pub fn verbose(&self) -> bool {
		self.verbose
	}

	pub fn solo_path(&self) -> &SPath {
		&self.solo_path
	}

	pub fn target_path(&self) -> &SPath {
		&self.target_path
	}
}

// region:    --- Froms

impl From<&SoloConfig> for AiSoloConfig {
	fn from(solo_config: &SoloConfig) -> Self {
		Self {
			verbose: solo_config.verbose(),
			solo_path: solo_config.solo_path().clone(),
			target_path: solo_config.target_path().clone(),
		}
	}
}

// endregion: --- Froms
