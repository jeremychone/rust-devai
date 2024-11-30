<div align="center">

<a href="https://crates.io/crates/devai"><img src="https://img.shields.io/crates/v/devai.svg" /></a>
<a href="https://github.com/jeremychone/rust-devai"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=DSuvkCHdD5I&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj"><img alt="Static Badge" src="https://img.shields.io/badge/YouTube_devai_Intro-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# **devai** - **Command Agent File Runner**

**Install** (for now, the simplest way to install is with cargo install)

- Install Rust: https://www.rust-lang.org/tools/install
- Run `cargo install devai`

**Usage**

```sh
# Init (optional; will be executed on each run as well)
devai init

# Will proofread and update the direct .md file from the current directory
devai run proof-read -f "./*.md"
# Can use multiple globs or direct files -f "./*.md" -f "./doc/**/*.md"
```

- How it works: 
  - It will run the installed Command Agent file `.devai/default/proof-read.devai` on all source files matching `./src/*.md` (Here is the source file for the default [proof-read.devai](/_init/agents/proof-read.devai))
    - Each matching file will become an `input` of type [FileRef](./_init/doc/lua.md#fileref) for the **Lua** and **Handlebars** parts of the agent file. 
  - An `.devai` Agent file is just a **Markdown File** with some sections for each stage of the agent processing. For example, an Agent file can have 
    - A `# Data` section with a ` ```lua ` code block to prepare the data, 
    - A `# Instruction` section that will be rendered with Handlebars templating, with access to `data` and `input`
    - A `# Output` section with a ` ```lua ` code block that has access to `data`, `input`, and `ai_response` 
  - Any `.devai` file can be an agent file and run with `devai run ./my-path/to/my-agent.devai`
    - Agents do not have to take any inputs or can generate their own in the `# Before All` section.
  - **Multi Stage** process, with the following stage (each stage is **optional**)

| Stage           | Language       | Description                                                                                                |
|-----------------|----------------|------------------------------------------------------------------------------------------------------------|
| `# Before All`  | **Lua**        | Reshape/generate inputs, and add command global data to scope (the "map" of the map/reduce capability).    |
| `# Data`        | **Lua**        | Gather additional data per input, and return it for the next stages.                                       |
| `# System`      | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                                |
| `# Instruction` | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                                |
| `# Assistant`   | **Handlebars** | Optional for special customizations, such as the "Jedi Mind Trick."                                        |
| `# Output`      | **Lua**        | Processes the `ai_response` from the LLM. Otherwise, `ai_response.content` will be output to the terminal. |
| `# After All`   | **Lua**        | Called with `inputs` and `outputs` for post-processing after all inputs are completed.                     |
<style>table td:first-child {white-space: nowrap;}</style>

**IMPORTANT**: Make sure everything is committed before usage.

# Key Concept - **One Agent** == **One Markdown** 

The main **devai** concept is to minimize the friction of creating and running an agent while providing maximum control over how we want those agents to run, and maximizing iteration speed to mature them quickly.

Here are some of the key **devai** characteristics. 

- **One Agent** == **One Markdown** 
    - (i.e., `my-agent.devai`, a `.devai` file is a markdown file with multi-stage sections described below)
- **Multi AI Provider / Models** Supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come. 
- **Lua** for Scripting.
- **Handlebars** for prompt templating. 


[more info on stages](_init/doc/README.md#complete-stages-description)

## [devai doc](_init/doc/README.md)

See the devai doc at **[_init/doc/README.md](_init/doc/README.md)**

You can also run the `ask-devai` agent. 

```sh
# IMPORTANT - Make sure you have the `OPENAI_API_KEY` or the key of your model in your environment
devai run ask-devai

# and then open the `.devai/tmp/ask-devai.md`
```

[more info on api keys](_init/doc/README.md#api-keys)

## Thanks

**TOP COLLABORATOR** Big **thanks** to [Stephan Philipakis](https://github.com/sphilipakis), a top **devai** collaborator contributing to the next-generation methodology for production coding with GenAI.


## Future Plan

- More Lua functions
- Agent module `my-module` may run `my-module/main.devai`, and running `my-module/some-other` will run `my-module/some-other.devai`
- Support Lua `Require`
- Full TUI/Ratatui 
- Split runtime to [agentic](https://crates.io/crates/agentic)
