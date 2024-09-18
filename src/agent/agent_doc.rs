use crate::agent::Agent;
use crate::Result;
use simple_fs::read_to_string;
use std::path::Path;

#[derive(Debug)]
pub struct AgentDoc {
	raw_content: String,
}

/// Constructor
impl AgentDoc {
	pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let raw_content = read_to_string(path)?;
		Ok(Self { raw_content })
	}

	pub fn into_agent(self) -> Result<Agent> {
		#[derive(Debug)]
		enum CaptureMode {
			None,
			// below the data heading (perhaps not is code block)
			DataSection,
			// inside the code block
			DataCodeBlock,
			Inst,
			// below the output heading (perhaps not is code block)
			OutputSection,
			// inside the code block
			OutputCodeBlock,
		}

		let mut capture_mode = CaptureMode::None;

		let mut inst = String::new();
		let mut data_script = String::new();
		let mut output_script = String::new();

		// -- The actual parsing
		// NOTE: For now custom parsing. `markdown` and `pulldown-cmark` are loosing information
		//       and therefore not appropriate for this usecase
		for line in self.raw_content.lines() {
			// if heading we decide the capture mode
			if line.starts_with('#') && !line.starts_with("##") {
				let header = line[1..].trim().to_lowercase();
				if header == "data" {
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

		let agent = Agent {
			inst,
			data_script: string_as_option_if_empty(data_script),
			output_script: string_as_option_if_empty(output_script),
			messages: None,
		};

		Ok(agent)
	}
}

// region:    --- String Support

fn push_line(content: &mut String, line: &str) {
	content.push_str(line);
	// only add new line if line is not empty.
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

	#[test]
	fn test_agent_doc_demo_ok() -> Result<()> {
		// -- Setup & Fixtures
		let agent_doc_path = "./tests-data/agents/agent-demo.md";

		// -- Exec
		let doc = AgentDoc::from_file(agent_doc_path)?;
		let agent = doc.into_agent()?;

		// -- Check
		assert!(agent.inst.contains("Some paragraph for instruction"), "instruction");
		let data_script = agent.data_script.ok_or("Should have data_script")?;
		assert!(
			data_script.contains("// Some scripts that load the data"),
			"data_script"
		);
		let output_script = agent.output_script.ok_or("Should have output_script")?;
		assert!(
			output_script.contains("/// optional output processing."),
			"output_script"
		);

		Ok(())
	}
}

// endregion: --- Tests
