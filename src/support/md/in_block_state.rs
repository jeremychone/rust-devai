pub enum InBlockState {
	In3,
	In6,
	Out,
}

impl InBlockState {
	pub fn is_out(&self) -> bool {
		matches!(self, InBlockState::Out)
	}

	pub fn compute_new(self, line: &str) -> Self {
		if !line.starts_with("```") {
			return self;
		}
		let is_6 = line.starts_with("``````");

		match self {
			InBlockState::In3 => {
				// toggle out only if same (if not 6, it's 3)
				if !is_6 {
					InBlockState::Out
				} else {
					self
				}
			}
			InBlockState::In6 => {
				// toggle out only if same
				if is_6 {
					InBlockState::Out
				} else {
					self
				}
			}
			InBlockState::Out => {
				if is_6 {
					InBlockState::In6
				} else {
					InBlockState::In3
				}
			}
		}
	}
}
