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
	/// Initialize the `.devai/` folder with the base setting files. Any file that already exists will not be touched.
	Init,

	/// Executes the Command Agent 'markdown' based on its name or short name (`proof-comments.md` will match `proof-comments` or `pc`)
	Run(RunArgs),

	/// New
	New(NewArgs),

	/// List the available command agents
	List,
}

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct RunArgs {
	/// The name of the Command Agent to execute, required.
	/// This should be the name of the markdown file under `.devai/customs` or `.devai/defaults` (without extension),
	/// or the filename initial `proof-comments.md` will match to `proof-comments` or `pc`
	pub cmd_agent_name: String,

	/// Optional file parameter, allowing multiple files
	#[arg(short = 'f', long = "on-files")]
	pub on_files: Option<Vec<String>>,

	/// Optional watch flag
	#[arg(short = 'w', long = "watch")]
	pub watch: bool,

	/// Verbose mode
	#[arg(short = 'v', long = "verbose")]
	pub verbose: bool,

	/// Dry mode, takes either 'req' or 'res'
	#[arg(long = "dry", value_parser = ["req", "res"])]
	pub dry_mode: Option<String>,
}

/// Arguments for the `run` subcommand
#[derive(Parser, Debug)]
pub struct NewArgs {
	/// The command agent name which will be created under
	/// e.g., `devai new my-cool-agent`
	///        will create `.devai/custom/command-agent/my-cool-agent.devai`
	pub agent_path: String,
}
