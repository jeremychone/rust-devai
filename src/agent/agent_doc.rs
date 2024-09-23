use crate::agent::agent_config::AgentConfig;
use crate::agent::{Agent, AgentInner};
use crate::support::tomls::parse_toml;
use crate::Result;
use genai::ModelName;
use simple_fs::{read_to_string, SFile};
use std::path::Path;

#[derive(Debug)]
pub struct AgentDoc {
	sfile: SFile,
	raw_content: String,
}

/// Constructor
impl AgentDoc {
	pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let sfile = SFile::new(path)?;
		let raw_content = read_to_string(path)?;
		Ok(Self { raw_content, sfile })
	}

	pub fn into_agent(self, mut config: AgentConfig) -> Result<Agent> {
		#[derive(Debug)]
		enum CaptureMode {
			None,

			// Bellow the # Config section
			ConfigSection,
			// inside the ConfigTomlBlock
			ConfigTomlBlock,

			// Below the data heading (perhaps not in a code block)
			DataSection,
			// Inside the code block
			DataCodeBlock,

			Inst,
			// Below the output heading (perhaps not in a code block)
			OutputSection,
			// Inside the code block
			OutputCodeBlock,
		}

		impl CaptureMode {
			fn is_inside_code_block(&self) -> bool {
				matches!(
					self,
					CaptureMode::ConfigTomlBlock | CaptureMode::DataCodeBlock | CaptureMode::OutputCodeBlock
				)
			}
		}

		let mut capture_mode = CaptureMode::None;

		let mut config_toml = String::new();
		let mut data_script = String::new();
		let mut inst = String::new();
		let mut output_script = String::new();

		// -- The actual parsing
		// NOTE: For now custom parsing. `markdown` and `pulldown-cmark` are losing information
		//       and therefore not appropriate for this use case
		for line in self.raw_content.lines() {
			// If heading we decide the capture mode
			if !capture_mode.is_inside_code_block() && line.starts_with('#') && !line.starts_with("##") {
				let header = line[1..].trim().to_lowercase();
				if header == "config" {
					capture_mode = CaptureMode::ConfigSection;
				} else if header == "data" {
					capture_mode = CaptureMode::DataSection;
				} else if header == "inst" || header == "instruction" {
					capture_mode = CaptureMode::Inst;
				} else if header == "output" {
					capture_mode = CaptureMode::OutputSection;
				} else {
					// Stop processing current section if new top-level header
					capture_mode = CaptureMode::None;
				}
				continue;
			}

			match capture_mode {
				CaptureMode::None => {}

				CaptureMode::ConfigSection => {
					if line.starts_with("```toml") {
						capture_mode = CaptureMode::ConfigTomlBlock;
						continue;
					}
				}

				CaptureMode::ConfigTomlBlock => {
					if line.starts_with("```") {
						capture_mode = CaptureMode::None;
						continue;
					} else {
						push_line(&mut config_toml, line);
					}
				}

				CaptureMode::DataSection => {
					if line.starts_with("```rhai") {
						capture_mode = CaptureMode::DataCodeBlock;
						continue;
					}
				}
				CaptureMode::DataCodeBlock => {
					if line.starts_with("```") {
						capture_mode = CaptureMode::None;
						continue;
					} else {
						push_line(&mut data_script, line);
					}
				}
				CaptureMode::Inst => {
					push_line(&mut inst, line);
				}
				CaptureMode::OutputSection => {
					if line.starts_with("```rhai") {
						capture_mode = CaptureMode::OutputCodeBlock;
						continue;
					}
				}
				CaptureMode::OutputCodeBlock => {
					if line.starts_with("```") {
						capture_mode = CaptureMode::None;
						continue;
					} else {
						push_line(&mut output_script, line);
					}
				}
			}
		}

		// -- Returning the data
		if !config_toml.is_empty() {
			let value = parse_toml(&config_toml)?;
			config = config.merge(value)?;
		}

		let genai_model_name = config.model().map(ModelName::from);

		let agent = AgentInner {
			config,

			name: self.sfile.file_stem().to_string(),
			file_name: self.sfile.file_name().to_string(),
			file_path: self.sfile.to_str().to_string(),

			genai_model_name,

			inst,
			data_script: string_as_option_if_empty(data_script),
			output_script: string_as_option_if_empty(output_script),
			messages: None,
		};

		Ok(agent.into())
	}
}

// region:    --- String Support

fn push_line(content: &mut String, line: &str) {
	content.push_str(line);
	// Only add a new line if the line is not empty.
	// Otherwise, it was a new line, no need to add another one
	if !line.trim().is_empty() {
		content.push('\n');
	}
}

fn string_as_option_if_empty(content: String) -> Option<String> {
	if content.is_empty() {
		None
	} else {
		Some(content)
	}
}

// endregion: --- String Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::test_support::default_agent_config_for_test;

	#[test]
	fn test_agent_doc_demo_ok() -> Result<()> {
		// -- Setup & Fixtures
		let agent_doc_path = "./tests-data/agents/agent-demo.md";

		// -- Exec
		let doc = AgentDoc::from_file(agent_doc_path)?;
		let agent = doc.into_agent(default_agent_config_for_test())?;

		// -- Check
		assert!(agent.inst().contains("Some paragraph for instruction"), "instruction");
		let data_script = agent.data_script().ok_or("Should have data_script")?;
		assert!(
			data_script.contains("// Some scripts that load the data"),
			"data_script"
		);
		let output_script = agent.output_script().ok_or("Should have output_script")?;
		assert!(
			output_script.contains("/// Optional output processing."),
			"output_script does not contain."
		);

		Ok(())
	}

	#[test]
	fn test_agent_doc_config_ok() -> Result<()> {
		// -- Setup & Fixtures
		let agent_doc_path = "./tests-data/agents/agent-demo.md";

		// -- Exec
		let doc = AgentDoc::from_file(agent_doc_path)?;
		let agent = doc.into_agent(default_agent_config_for_test())?;

		// -- Check config
		assert_eq!(agent.config().model(), Some("test_model_for_demo"));
		assert_eq!(agent.config().items_concurrency(), Some(8));

		// -- Check Other
		assert!(agent.inst().contains("Some paragraph for instruction"), "instruction");
		let data_script = agent.data_script().ok_or("Should have data_script")?;
		assert!(
			data_script.contains("// Some scripts that load the data"),
			"data_script"
		);
		let output_script = agent.output_script().ok_or("Should have output_script")?;
		assert!(
			output_script.contains("/// Optional output processing."),
			"output_script does not contain."
		);

		Ok(())
	}
}

// endregion: --- Tests
