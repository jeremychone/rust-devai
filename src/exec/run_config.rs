use crate::cli::RunArgs;

pub struct RunConfig {
	cmd_agent: String,
	on_file_globs: Option<Vec<String>>,
}

impl RunConfig {
	pub fn cmd_agent(&self) -> &str {
		&self.cmd_agent
	}

	pub fn on_file_globs(&self) -> Option<Vec<&str>> {
		self.on_file_globs.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect())
	}
}

// region:    --- From AppArgs

impl From<RunArgs> for RunConfig {
	fn from(args: RunArgs) -> Self {
		// -- When a simple name is provided
		let on_file_globs = if let Some(on_files) = args.on_files {
			let on_files_globs = on_files
				.into_iter()
				.map(|s| {
					// The goal of this logic is to make a simple name into a wider glob so that the user does not have to specify the exact file name.
					// TODO: This branch can be improved to handle any absolute or relative path.
					if s.contains('*') || s.starts_with("./") || s.starts_with("/") {
						s
					} else {
						format!("**/{s}")
					}
				})
				.collect::<Vec<String>>();
			Some(on_files_globs)
		} else {
			None
		};

		Self {
			cmd_agent: args.cmd_agent_name,
			on_file_globs,
		}
	}
}

// endregion: --- From AppArgs
