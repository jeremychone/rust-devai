use crate::agent::agent_config::AgentConfig;
use crate::agent::{Agent, AgentInner, PartKind, PromptPart};
use crate::support::tomls::parse_toml;
use crate::Result;
use genai::ModelName;
use simple_fs::{read_to_string, SPath};
use std::path::Path;

#[derive(Debug)]
pub struct AgentDoc {
	spath: SPath,
	raw_content: String,
}

/// Constructor
impl AgentDoc {
	pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
		let spath = SPath::new(path.as_ref())?;
		let raw_content = read_to_string(path)?;
		Ok(Self { spath, raw_content })
	}

	#[cfg(test)]
	pub fn from_content(path: impl AsRef<Path>, content: impl Into<String>) -> Result<Self> {
		let spath = SPath::new(path.as_ref())?;
		let raw_content = content.into();
		Ok(Self { spath, raw_content })
	}

	pub fn into_agent(self, name: impl Into<String>, config: AgentConfig) -> Result<Agent> {
		let agent_inner = self.into_agent_inner(name.into(), config)?;
		let agent = Agent::new(agent_inner)?;
		Ok(agent)
	}

	/// Internal method to create the first part of the agent inner
	/// This is sort of a Lexer, but very customize to extracting the Agent parts
	fn into_agent_inner(self, name: String, mut config: AgentConfig) -> Result<AgentInner> {
		#[derive(Debug)]
		enum CaptureMode {
			None,

			// Below the output heading (perhaps not in a code block)
			BeforeAllSection,
			// Inside the code block
			BeforeAllCodeBlock,

			// Bellow the # Config section
			ConfigSection,
			// inside the ConfigTomlBlock
			ConfigTomlBlock,

			// Below the data heading (perhaps not in a code block)
			DataSection,
			// Inside the code block
			DataCodeBlock,

			PromptPart,

			// Below the output heading (perhaps not in a code block)
			OutputSection,
			// Inside the code block
			OutputCodeBlock,

			// Below the output heading (perhaps not in a code block)
			AfterAllSection,
			// Inside the code block
			AfterAllCodeBlock,
		}

		impl CaptureMode {
			/// Here we are inside a code block that is getting captured for an action
			/// either rhai script, toml, ...
			///
			/// NOTE: This is not used anymore since we have the `is_in_any_block`, but can be later.
			#[allow(unused)]
			fn is_inside_actionable_block(&self) -> bool {
				matches!(
					self,
					CaptureMode::ConfigTomlBlock
						| CaptureMode::BeforeAllCodeBlock
						| CaptureMode::DataCodeBlock
						| CaptureMode::OutputCodeBlock
						| CaptureMode::AfterAllCodeBlock
				)
			}
		}

		let mut capture_mode = CaptureMode::None;

		let mut config_toml = String::new();
		let mut before_all_script = String::new();
		let mut data_script = String::new();
		let mut output_script = String::new();
		let mut after_all_script = String::new();

		let mut prompt_parts: Vec<PromptPart> = Vec::new();
		// the vec String allow to be more efficient (as join later is more efficient)
		let mut current_part: Option<CurrentPromptPart> = None;

		// -- The actual parsing
		// NOTE: Need custom parser/lexer given the nature of the agent format.
		//       Markdown parsers tend to be lossless and would need wuite a bit of extra post-processing anyway.
		//       So, here we do one path, and capture what we need, exactly the way we need it

		// This is a simple flag that toggle on and off when entering a code block
		enum BlockState {
			In3,
			In6,
			Out,
		}

		impl BlockState {
			fn compute_new(self, line: &str) -> Self {
				if !line.starts_with("```") {
					return self;
				}
				let is_6 = line.starts_with("``````");

				match self {
					BlockState::In3 => {
						// toggle out only if same (if not 6, it's 3)
						if !is_6 {
							BlockState::Out
						} else {
							self
						}
					}
					BlockState::In6 => {
						// toggle out only if same
						if is_6 {
							BlockState::Out
						} else {
							self
						}
					}
					BlockState::Out => {
						if is_6 {
							BlockState::In6
						} else {
							BlockState::In3
						}
					}
				}
			}
		}

		let mut block_state = BlockState::Out;

		for line in self.raw_content.lines() {
			// TODO needs to support the
			block_state = block_state.compute_new(line);

			// If heading we decide the capture mode
			if matches!(block_state, BlockState::Out) && line.starts_with('#') && !line.starts_with("##") {
				let header = line[1..].trim().to_lowercase();
				if header == "config" {
					capture_mode = CaptureMode::ConfigSection;
				} else if header == "before all" {
					capture_mode = CaptureMode::BeforeAllSection;
				} else if header == "data" {
					capture_mode = CaptureMode::DataSection;
				} else if header == "output" {
					capture_mode = CaptureMode::OutputSection;
				} else if header == "after all" {
					capture_mode = CaptureMode::AfterAllSection;
				} else if let Some(part_kind) = get_prompt_part_kind(&header) {
					capture_mode = CaptureMode::PromptPart;
					// we finalize the previous part if present
					finalize_current_prompt_part(&mut current_part, &mut prompt_parts);
					// then, we create the new current_part
					current_part = Some(CurrentPromptPart(part_kind, Vec::new()));
				} else {
					// Stop processing current section if new top-level header
					capture_mode = CaptureMode::None;
				}
				continue;
			}

			match capture_mode {
				CaptureMode::None => {}

				// -- Config
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

				// -- Before All
				CaptureMode::BeforeAllSection => {
					if line.starts_with("```rhai") {
						capture_mode = CaptureMode::BeforeAllCodeBlock;
						continue;
					}
				}
				CaptureMode::BeforeAllCodeBlock => {
					if line.starts_with("```") {
						capture_mode = CaptureMode::None;
						continue;
					} else {
						push_line(&mut before_all_script, line);
					}
				}

				// -- Data
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

				// -- Pompt Part
				CaptureMode::PromptPart => {
					if let Some(current_part) = &mut current_part {
						current_part.1.push(line);
					} else {
						// This should not happen, as the current_part should be been created when we enterred the section
						// TODO: Need to capture warning if we reach this point.
					}
				}

				// -- Output
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

				// -- After All
				CaptureMode::AfterAllSection => {
					if line.starts_with("```rhai") {
						capture_mode = CaptureMode::AfterAllCodeBlock;
						continue;
					}
				}
				CaptureMode::AfterAllCodeBlock => {
					if line.starts_with("```") {
						capture_mode = CaptureMode::None;
						continue;
					} else {
						push_line(&mut after_all_script, line);
					}
				}
			}
		}

		// -- We finilize the last part if it was not closed
		finalize_current_prompt_part(&mut current_part, &mut prompt_parts);

		// -- Returning the data
		if !config_toml.is_empty() {
			let value = parse_toml(&config_toml)?;
			config = config.merge(value)?;
		}

		let genai_model_name = config.model().map(ModelName::from);

		let agent_inner = AgentInner {
			config,

			name,

			file_name: self.spath.name().to_string(),
			file_path: self.spath.to_str().to_string(),

			genai_model_name,

			before_all_script: string_as_option_if_empty(before_all_script),
			data_script: string_as_option_if_empty(data_script),

			prompt_parts,

			output_script: string_as_option_if_empty(output_script),
			after_all_script: string_as_option_if_empty(after_all_script),
		};

		Ok(agent_inner)
	}
}

// region:    --- Support

fn get_prompt_part_kind(header: &str) -> Option<PartKind> {
	if header == "inst" || header == "instruction" {
		Some(PartKind::Instruction)
	} else if header == "system" {
		Some(PartKind::System)
	} else if header == "assistant" || header == "model" || header == "mind trick" || header == "jedi trick" {
		Some(PartKind::Assistant)
	} else {
		None
	}
}

/// Type of the function below and the `into_agent_inner` lexer
struct CurrentPromptPart<'a>(PartKind, Vec<&'a str>);

/// Finalize a eventual current_part
fn finalize_current_prompt_part(current_part: &mut Option<CurrentPromptPart<'_>>, prompt_parts: &mut Vec<PromptPart>) {
	if let Some(current_part) = current_part.take() {
		// to have the last line
		let kind = current_part.0;
		let mut content = current_part.1;
		content.push("");
		let content = content.join("\n");

		let part = PromptPart { kind, content };
		prompt_parts.push(part);
	}
}

/// Push a new line and the a \n to respect the new line
fn push_line(content: &mut String, line: &str) {
	content.push_str(line);
	content.push('\n');
}

fn string_as_option_if_empty(content: String) -> Option<String> {
	if content.is_empty() {
		None
	} else {
		Some(content)
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, default_agent_config_for_test};

	#[test]
	fn test_agent_doc_demo_ok() -> Result<()> {
		// -- Setup & Fixtures
		let agent_doc_path = "./tests-data/agent-doc/agent-demo.md";

		// -- Exec
		let doc = AgentDoc::from_file(agent_doc_path)?;
		let agent = doc.into_agent(agent_doc_path, default_agent_config_for_test())?;

		// -- Check
		let &first_prompt_part = agent.prompt_parts().first().ok_or("Should have a prompt part")?;
		let inst = &first_prompt_part.content;
		assert_contains(inst, "Some paragraph for instruction");
		assert_contains(inst, "- Two");
		assert_contains(inst, "block-01");
		assert_contains(inst, "block-02");
		assert_contains(inst, "# block-03");
		assert_contains(inst, "# Instruction");
		let data_script = agent.data_script().ok_or("Should have data_script")?;
		assert_contains(data_script, "// Some scripts that load the data");
		let output_script = agent.output_script().ok_or("Should have output_script")?;
		assert_contains(output_script, "/// Optional output processing.");

		Ok(())
	}

	#[test]
	fn test_agent_doc_config_ok() -> Result<()> {
		// -- Setup & Fixtures
		let agent_doc_path = "./tests-data/agent-doc/agent-demo.md";

		// -- Exec
		let doc = AgentDoc::from_file(agent_doc_path)?;
		let agent = doc.into_agent(agent_doc_path, default_agent_config_for_test())?;

		// -- Check config
		assert_eq!(agent.config().model(), Some("test_model_for_demo"));
		assert_eq!(agent.config().inputs_concurrency(), Some(8), "concurrency");

		// -- Check Other
		let &first_prompt_part = agent.prompt_parts().first().ok_or("Should have a prompt part")?;
		let inst = &first_prompt_part.content;
		assert_contains(inst, "Some paragraph for instruction");
		let data_script = agent.data_script().ok_or("Should have data_script")?;
		assert_contains(data_script, "// Some scripts that load the data");
		let output_script = agent.output_script().ok_or("Should have output_script")?;
		assert_contains(output_script, "/// Optional output processing.");

		Ok(())
	}

	#[test]
	fn test_agent_doc_all_sections_ok() -> Result<()> {
		// -- Setup & Fixtures
		let agent_doc_path = "./tests-data/agent-doc/agent-all-sections.md";

		// -- Exec
		let doc = AgentDoc::from_file(agent_doc_path)?;
		let agent = doc.into_agent(agent_doc_path, default_agent_config_for_test())?;

		// -- Check config
		assert_eq!(agent.config().model(), Some("test_model_for_demo"));
		assert_eq!(agent.config().inputs_concurrency(), None);

		let &first_prompt_part = agent.prompt_parts().first().ok_or("Should have a prompt part")?;

		// -- Check Sections
		assert_contains(
			agent.before_all_script().ok_or("No before_all script")?,
			"let before_all = \"before_all\";\n",
		);

		assert_contains(
			agent.data_script().ok_or("No data script")?,
			"let some_data = \"Some Data\";\nreturn some_data;\n",
		);

		assert_contains(&first_prompt_part.content, "\nSome instruction\n\n");

		assert_contains(
			agent.data_script().ok_or("Data Script missing")?,
			"let some_data = \"Some Data\";\nreturn some_data;\n",
		);

		assert_contains(
			agent.output_script().ok_or("No output script")?,
			"let some_output = \"Some Output\";\n",
		);

		assert_contains(
			agent.after_all_script().ok_or("No after all script")?,
			"let after_all = \"after_all\";\n",
		);

		Ok(())
	}
}

// endregion: --- Tests
