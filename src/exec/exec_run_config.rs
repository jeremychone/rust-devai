use crate::cli::RunArgs;

/// The Dry mode of the content.
///
/// > Note: Might want to move this out of the exec sub module as it is used in ai one (code-clean)
#[derive(Debug, Clone)]
pub enum DryMode {
	Req,
	Res,
	None, // not dry mode
}

pub struct ExecRunConfig {
	watch: bool,
	verbose: bool,
	dry_mode: DryMode,
	cmd_agent: String,
	on_file_globs: Option<Vec<String>>,
}

impl ExecRunConfig {
	pub fn cmd_agent(&self) -> &str {
		&self.cmd_agent
	}

	pub fn on_file_globs(&self) -> Option<Vec<&str>> {
		self.on_file_globs.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect())
	}

	pub fn watch(&self) -> bool {
		self.watch
	}

	pub fn verbose(&self) -> bool {
		self.verbose
	}

	pub fn dry_mode(&self) -> DryMode {
		self.dry_mode.clone()
	}
}

// region:    --- From AppArgs

impl From<RunArgs> for ExecRunConfig {
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

		let dry_mode = match args.dry_mode.as_deref() {
			Some("req") => DryMode::Req,
			Some("res") => DryMode::Res,
			_ => DryMode::None,
		};

		Self {
			verbose: args.verbose,
			watch: args.watch,
			dry_mode,
			cmd_agent: args.cmd_agent_name,
			on_file_globs,
		}
	}
}

// endregion: --- From AppArgs
