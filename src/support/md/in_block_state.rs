#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InBlockState {
	In3,
	In4,
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
		let is_4 = line.starts_with("````");

		match self {
			InBlockState::In3 => {
				// toggle out only if same (if not 6, it's 3)
				if !is_4 { InBlockState::Out } else { self }
			}
			InBlockState::In4 => {
				// toggle out only if same
				if is_4 { InBlockState::Out } else { self }
			}
			InBlockState::Out => {
				if is_4 {
					InBlockState::In4
				} else {
					InBlockState::In3
				}
			}
		}
	}
}
