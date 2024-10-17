use crate::exec::SoloConfig;
use simple_fs::SPath;

#[derive(Debug, Clone)]
pub struct AiSoloConfig {
	verbose: bool,
	target_path: SPath,
}

impl AiSoloConfig {
	pub fn verbose(&self) -> bool {
		self.verbose
	}

	pub fn target_path(&self) -> &SPath {
		&self.target_path
	}
}

/// Constructor
impl AiSoloConfig {
	/// Just for text for now.
	/// For normal path, the from SoloConfig is used
	#[cfg(test)]
	pub fn from_target_path(target_path: &str) -> crate::Result<Self> {
		Ok(Self {
			verbose: false,
			target_path: SPath::new(target_path)?,
		})
	}
}

// region:    --- Froms

impl From<&SoloConfig> for AiSoloConfig {
	fn from(solo_config: &SoloConfig) -> Self {
		Self {
			verbose: solo_config.verbose(),
			target_path: solo_config.target_path().clone(),
		}
	}
}

// endregion: --- Froms
