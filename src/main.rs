// region:    --- Modules

mod agent;
mod ai;
mod error;
mod script;
mod support;
mod tmpl;

use crate::agent::get_agent_instruction;
use crate::ai::run_ai_on_file;
use crate::support::args::AppArgs;
use crate::support::cred::get_or_prompt_api_key;
use clap::Parser;
pub use error::{Error, Result};
use simple_fs::list_files;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- command arguments
	let mut args = AppArgs::parse().args;
	args.reverse();

	// -- get the cmd
	// get the agent command
	let cmd = args.pop().ok_or("Cannot find cmd")?;
	// get the eventual name match
	let target_glob = match args.pop() {
		Some(target) => {
			if target.contains('*') {
				target
			} else {
				format!("**/{target}")
			}
		}
		None => "**/*.rs".to_string(),
	};

	let agent_inst = get_agent_instruction(&cmd)?;

	// -- get ai client
	let client = ai::get_genai_client()?;

	// for now, very simple
	let is_glob = target_glob.contains("*");

	let src_files = list_files("./", Some(&[target_glob.as_ref()]), None)?;

	for src_file in src_files {
		println!("processing: {src_file}");
		let source = std::fs::read_to_string(&src_file)?;
		run_ai_on_file(&client, &agent_inst, src_file).await?;
	}

	Ok(())
}
