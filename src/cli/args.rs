use clap::{command, Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AppArgs {
	/// Subcommands
	#[command(subcommand)]
	pub cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	/// Run subcommand
	#[command(
		about = "Executes the Command Agent 'markdown' based on its name or short name (`proof-comments.md` will match `proof-comments` or `pc`)"
	)]
	Run(RunArgs),

	/// Initialize the `.devai/` folder with the base setting files. Any file that already exists will not be touched.
	Init,
}

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct RunArgs {
	/// The name of the Command Agent to execute, required.
	/// This should be the name of the markdown file under `.devai/customs` or `.devai/defaults` (without extension),
	/// or thhe filename initial `proof-comments.md` will match to `proof-comments` or `pc`
	pub cmd_agent_name: String,

	/// Optional file parameter, allowing multiple files
	#[arg(short = 'f', long = "on-files")]
	pub on_files: Option<Vec<String>>,
}
