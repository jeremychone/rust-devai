use clap::{command, Parser};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AppArgs {
	/// Subcommands
	pub cmd: String,

	/// List of positional arguments
	pub args: Vec<String>,

	/// Optional file parameter, allowing multiple files
	#[arg(short = 'f', long = "on-files")]
	pub on_files: Option<Vec<String>>,
}
