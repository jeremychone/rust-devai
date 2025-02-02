<div align="center">

<a href="https://crates.io/crates/devai"><img src="https://img.shields.io/crates/v/devai.svg" /></a>
<a href="https://github.com/jeremychone/rust-devai"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj"><img alt="Static Badge" src="https://img.shields.io/badge/YouTube_devai_Intro-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# **devai** - **Command Agent File Runner**

- Website: https://devai.run

- [Full intro video for v0.5](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)

- Top new features:
  - 2025-01-27 (v0.5.9) - Deepseek distill models support for Groq and Ollama (local)
  - 2025-01-23 (v0.5.7) - `devai run craft/text` or `devai run craft/code` (example of cool new agent module support)
  - 2025-01-06 (v0.5.4) - DeepSeek `deepseek-chat` support
  - 2024-12-08 (v0.5.1) - Added support for **xAI**

- Thanks to
  - [David Horner](https://github.com/davehorner) for adding Windows support for Open Agent (with VSCode) ([#30](https://github.com/jeremychone/rust-devai/pull/30))
  - [Diaa Kasem](https://github.com/diaakasem) for `--non-interactive`/`--ni` mode ([#28](https://github.com/jeremychone/rust-devai/pull/28))
  - [Stephane Philipakis](https://github.com/sphilipakis), a top **devai** collaborator.

#### Install

_For now, the simplest way to install is with `cargo install`._

- Install Rust: https://www.rust-lang.org/tools/install
- Run `cargo install devai`

#### Usage

```sh
# Init (optional; will be executed on each run as well)
devai init

# Will proofread and update the direct .md file from the current directory
devai run proof-read -f "./*.md"
# Can use multiple globs or direct files -f "./*.md" -f "./doc/**/*.md"

# For a one-shot run (or --ni)
devai run --non-interactive proof-read -f "./doc/README.md"

```

The main concept of **devai** is to minimize friction in creating and running agents while providing maximum control over how we want those agents to run and maximizing iteration speed to mature them quickly.

**IMPORTANT 1**: Make sure everything is committed before usage (at least while you are learning about devai).

**IMPORTANT 2**: Make sure to have your **`OPENAI_API_KEY`**, **`ANTHROPIC_API_KEY`**, **`DEEPSEEK_API_KEY`**, or **`XAI_API_KEY`**, or the key of your model provider [more info on API keys](_init/doc/README.md#api-keys)

**NOTE**: Since `v0.5.4`, the agent folders now have the `command-` prefix under `.devai/` (DevAI will update the folder names when needed).

#### How it works

- **One Agent** == **One Markdown** 
    - A `.devai` Agent file is just a **Markdown File** with sections for each stage of the agent processing.
    - See below for all the [possible stages](#multi-stage).
- `devai run proof-read -f "./*.md"` will run the installed Command Agent file `.devai/default/proof-read.devai` on all source files matching `./src/*.md` (Here is the source file for the default [proof-read.devai](/_init/agents/proof-read.devai))
  - Each matching file will become an `input` of type [FileMeta](./_init/doc/lua.md#filemeta) for the **Lua** and **Handlebars** parts of the agent file. 
- **devai** agents are simple `.devai` files that can be placed anywhere on disk.
  - e.g., `devai run ./my-path/to/my-agent.devai ...`  
- **Multi AI Provider / Models** - **devai** uses [genai](https://crates.io/crates/genai) and therefore supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come. 
- **Lua** is used for all scripting (thanks to the great [mlua](https://crates.io/crates/mlua) crate).
- **Handlebars** is used for all prompt templating (thanks to the great Rust native [handlebars](https://crates.io/crates/handlebars) crate).     

### Multi Stage

A single **devai** file may comprise any of the following stages. 

| Stage           | Language       | Description                                                                                                |
|-----------------|----------------|------------------------------------------------------------------------------------------------------------|
| `# Before All`  | **Lua**        | Reshape/generate inputs and add command global data to scope (the "map" of the map/reduce capability).    |
| `# Data`        | **Lua**        | Gather additional data per input and return it for the next stages.                                       |
| `# System`      | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                                |
| `# Instruction` | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                                |
| `# Assistant`   | **Handlebars** | Optional for special customizations, such as the "Jedi Mind Trick."                                        |
| `# Output`      | **Lua**        | Processes the `ai_response` from the LLM. Otherwise, `ai_response.content` will be output to the terminal. |
| `# After All`   | **Lua**        | Called with `inputs` and `outputs` for post-processing after all inputs are completed.                     |


- `# Before All` / `# After All` can be considered as the **map**/**reduce** of the agent, and these will be run before and after the input processing.

[more info on stages](_init/doc/README.md#complete-stages-description)

## [devai doc](_init/doc/README.md)

See the devai documentation at **[_init/doc/README.md](_init/doc/README.md)** (With [Lua modules doc](_init/doc/lua.md))

You can also run the `ask-devai` agent. 

```sh
# IMPORTANT - Make sure you have the `OPENAI_API_KEY` or the key of your model in your environment
devai run ask-devai

# and then open the `.devai/tmp/ask-devai.md`
```

## Future Plans

- More Lua functions
- Agent module `my-module` may run `my-module/main.devai`, and running `my-module/some-other` will run `my-module/some-other.devai`
- Support Lua `Require`
- Full TUI/Ratatui 
- Split runtime to [agentic](https://crates.io/crates/agentic)
