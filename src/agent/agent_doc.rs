use crate::Result;
use crate::agent::agent_options::AgentOptions;
use crate::agent::agent_ref::AgentRef;
use crate::agent::{Agent, AgentInner, PartKind, PromptPart};
use crate::support::md::InBlockState;
use crate::support::tomls::parse_toml;
use genai::ModelName;
use simple_fs::{SPath, read_to_string};
use std::path::Path;
use std::sync::Arc;

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

	pub fn into_agent(self, name: &str, agent_ref: AgentRef, options: AgentOptions) -> Result<Agent> {
		let agent_inner = self.into_agent_inner(name, agent_ref, options)?;
		let agent = Agent::new(agent_inner)?;
		Ok(agent)
	}

	/// Internal method to create the first part of the agent inner
	/// This is sort of a Lexer, but very customize to extracting the Agent parts
	fn into_agent_inner(self, name: &str, agent_ref: AgentRef, agent_options: AgentOptions) -> Result<AgentInner> {
		#[derive(Debug)]
		enum CaptureMode {
			None,

			// Below the output heading (perhaps not in a code block)
			BeforeAllSection,
			// Inside the code block
			BeforeAllCodeBlock,

			// Below the # Options section
			OptionsSection,
			OptionsTomlBlock,

			// (legacy) Below the # Config section
			ConfigSection,
			// (legacy) inside the ConfigTomlBlock
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
			/// either Lua script, toml, ...
			///
			/// NOTE: This is not used anymore since we have the `is_in_any_block`, but can be later.
			#[allow(unused)]
			fn is_inside_actionable_block(&self) -> bool {
				matches!(
					self,
					CaptureMode::ConfigTomlBlock
						| CaptureMode::OptionsTomlBlock
						| CaptureMode::BeforeAllCodeBlock
						| CaptureMode::DataCodeBlock
						| CaptureMode::OutputCodeBlock
						| CaptureMode::AfterAllCodeBlock
				)
			}
		}

		let mut capture_mode = CaptureMode::None;

		// -- The buffers
		let mut config_toml: Vec<&str> = Vec::new();
		let mut options_toml: Vec<&str> = Vec::new();
		let mut before_all_script: Vec<&str> = Vec::new();
		let mut data_script: Vec<&str> = Vec::new();
		let mut output_script: Vec<&str> = Vec::new();
		let mut after_all_script: Vec<&str> = Vec::new();

		let mut prompt_parts: Vec<PromptPart> = Vec::new();
		// the vec String allow to be more efficient (as join later is more efficient)
		let mut current_part: Option<CurrentPromptPart> = None;

		// -- The actual parsing
		// NOTE: Need custom parser/lexer given the nature of the agent format.
		//       Markdown parsers tend to be lossless and would need wuite a bit of extra post-processing anyway.
		//       So, here we do one path, and capture what we need, exactly the way we need it

		let mut block_state = InBlockState::Out;

		for line in self.raw_content.lines() {
			block_state = block_state.compute_new(line);
			// If heading we decide the capture mode
			if block_state.is_out() && line.starts_with('#') && !line.starts_with("##") {
				let header = line[1..].trim().to_lowercase();
				if header == "config" {
					capture_mode = CaptureMode::ConfigSection;
				} else if header == "options" {
					capture_mode = CaptureMode::OptionsSection;
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

				// -- Config (legacy, now use options)
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

				// -- Options
				CaptureMode::OptionsSection => {
					if line.starts_with("```toml") {
						capture_mode = CaptureMode::OptionsTomlBlock;
						continue;
					}
				}

				CaptureMode::OptionsTomlBlock => {
					if line.starts_with("```") {
						capture_mode = CaptureMode::None;
						continue;
					} else {
						push_line(&mut options_toml, line);
					}
				}

				// -- Before All
				CaptureMode::BeforeAllSection => {
					if line.starts_with("```lua") {
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
					if line.starts_with("```lua") {
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
					if line.starts_with("```lua") {
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
					if line.starts_with("```lua") {
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

		let config_toml = buffer_to_string(config_toml);
		let options_toml = buffer_to_string(options_toml);

		let agent_options_ov: Option<AgentOptions> = match (config_toml, options_toml) {
			(None, None) => None,
			(None, Some(options_toml)) => Some(AgentOptions::from_options_value(parse_toml(&options_toml)?)?),
			(Some(config_toml), None) => Some(AgentOptions::from_config_value(parse_toml(&config_toml)?)?),
			(Some(_), Some(_)) => {
				return Err("\
Agent .aipack file cannot have a '# Config' and '# Options' section.
Use the '# Options' section ('# Config' is not the legacy way to provides agent options)
"
				.into());
			}
		};

		let agent_options = match agent_options_ov {
			Some(agent_options_ov) => agent_options.merge(agent_options_ov)?,
			None => agent_options,
		};

		// -- Get the model name
		let model_name = agent_options.model().map(ModelName::from);

		// -- Build the AgentInner
		let agent_inner = AgentInner {
			agent_options: Arc::new(agent_options),

			name: name.to_string(),
			agent_ref,

			file_name: self.spath.name().to_string(),
			file_path: self.spath.to_str().to_string(),

			model_name,

			before_all_script: buffer_to_string(before_all_script),
			data_script: buffer_to_string(data_script),

			prompt_parts,

			output_script: buffer_to_string(output_script),
			after_all_script: buffer_to_string(after_all_script),
		};

		Ok(agent_inner)
	}
}

/// Constructor for test
#[cfg(test)]
impl AgentDoc {
	pub fn from_content(path: impl AsRef<Path>, content: impl Into<String>) -> Result<Self> {
		let spath = SPath::new(path.as_ref())?;
		let raw_content = content.into();
		Ok(Self { spath, raw_content })
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
fn push_line<'a, 'b, 'c: 'b>(content: &'a mut Vec<&'b str>, line: &'c str) {
	content.push(line);
	content.push("\n");
}

fn buffer_to_string(content: Vec<&str>) -> Option<String> {
	if content.is_empty() {
		None
	} else {
		Some(content.join(""))
	}
}
