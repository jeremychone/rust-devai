# IMPORTANT NOTICE - NOW AIPACK from v0.6.0 and on

- This project is rebranding to [aipack](https://aipack.ai), a more suitable name for its future development.

- This repo is now `https://github.com/aipack-ai/aipack`.

- **same codebase**, **same feature set**, **same licenses (MIT or Apache)**

- But now `aipack` centric, which is going to bring huge value for the users and community.

Now

```sh
aip run demo@proof-read

# "demo" is the namespace (installed in ~/.aipack-base/pack/installed/demo/proof-read)
# "proof_read" is the AI Pack, which has one entry point agent `main.aip`

# namespace@pack_name can be put in custom, as `~/.aipack-base/pack/custom/demo/proof-read`
# In this case, it will take precedence over the one form `installed`
```

You can find more information in the following [discussion #51](https://github.com/aipack-ai/aipack/discussions/51)

# aipack - Build, Run, Share

<div align="center">

<a href="https://crates.io/crates/aipack"><img src="https://img.shields.io/crates/v/aipack.svg" /></a>
<a href="https://github.com/jeremychone/rust-aipack"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj"><img alt="Static Badge" src="https://img.shields.io/badge/YouTube_aipack_Intro-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

- Website: https://aipack.ai

- [Full intro video for v0.5 (still old devai name, but same concept)](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)

- Built on top of the [Rust genai library](https://crates.io/crates/genai), which supports all the top AI providers and models (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Ollama, xAI, and Cohere).

- Top new features:
  - 2025-02-22 (v0.5.11) - Huge update with parametric agents, and coder (more info soon)
  - 2025-01-27 (v0.5.9) - Deepseek distill models support for Groq and Ollama (local)
  - 2025-01-23 (v0.5.7) - `aipack run craft/text` or `aipack run craft/code` (example of cool new agent module support)
  - 2025-01-06 (v0.5.4) - DeepSeek `deepseek-chat` support
  - 2024-12-08 (v0.5.1) - Added support for **xAI**

- **WINDOWS DISCLAIMER:**
    - This CLI uses a path scheme from Mac/Unix-like systems, which might not function correctly in the Windows `bat` command line.
    - Full Windows local path support is in development.
    - **RECOMMENDATION:** Use PowerShell or WSL on Windows. Please log issues if small changes can accommodate Windows PowerShell/WSL.

- Thanks to
  - **[Stephane Philipakis](https://github.com/sphilipakis)**, a **top** [aipack](https://aipack.ai) collaborator.
  - [David Horner](https://github.com/davehorner) for adding Windows support for Open Agent (with VSCode) ([#30](https://github.com/jeremychone/rust-aipack/pull/30))
  - [Diaa Kasem](https://github.com/diaakasem) for `--non-interactive`/`--ni` mode ([#28](https://github.com/jeremychone/rust-aipack/pull/28))

#### Install

_For now, the simplest way to install is with `cargo install aipack`._

- Install Rust: https://www.rust-lang.org/tools/install
- Run `cargo install aipack`

#### Usage

```sh
# Init
aipack init
# This will create `./.aipack/` on the current directory, making that directory a aipack workspace
#  (this will also have a  `./.aipack/custom/my/sample` AIPACK which can be ran with `aip run my@sample`)
#
# And, if not already created a `~/.aipack-base/` with the base/shared config.toml and 

# Then, to run you sample from your workspace.
aip run my@sample

# Or run some demo packs


# Can use multiple globs or direct files -f "./*.md" -f "./doc/**/*.md"

# For a one-shot run (or --ni)
aip run demo@proof-read -f "./doc/README.md" --non-interactive 

```

The main concept of **aipack** is to minimize friction in creating and running agents while providing maximum control over how we want those agents to run and maximizing iteration speed to mature them quickly.

**IMPORTANT 1**: Make sure everything is committed before usage (at least while you are learning about aipack).

**IMPORTANT 2**: Make sure to have your **`OPENAI_API_KEY`**, **`ANTHROPIC_API_KEY`**, **`DEEPSEEK_API_KEY`**, or **`XAI_API_KEY`**, or the key of your model provider [more info on API keys](_init/doc/README.md#api-keys)

**NOTE**: Since `v0.5.4`, the agent folders now have the `command-` prefix under `.aipack/` (aipack will update the folder names when needed).

#### How it works

- **One Agent** == **One Markdown** 
    - A `.aipack` Agent file is just a **Markdown File** with sections for each stage of the agent processing.
    - See below for all the [possible stages](#multi-stage).
- `aipack run proof-read -f "./*.md"` will run the installed Command Agent file `.aipack/default/proof-read.aipack` on all source files matching `./src/*.md` (Here is the source file for the default [proof-read.aipack](/_init/agents/proof-read.aipack))
  - Each matching file will become an `input` of type [FileMeta](./_init/doc/lua.md#filemeta) for the **Lua** and **Handlebars** parts of the agent file. 
- **aipack** agents are simple `.aipack` files that can be placed anywhere on disk.
  - e.g., `aipack run ./my-path/to/my-agent.aipack ...`  
- **Multi AI Provider / Models** - **aipack** uses [genai](https://crates.io/crates/genai) and therefore supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come. 
- **Lua** is used for all scripting (thanks to the great [mlua](https://crates.io/crates/mlua) crate).
- **Handlebars** is used for all prompt templating (thanks to the great Rust native [handlebars](https://crates.io/crates/handlebars) crate).     

### Multi Stage

A single **aipack** file may comprise any of the following stages. 

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

## [aipack doc](_init/doc/README.md)

See the aipack documentation at **[_init/doc/README.md](_init/doc/README.md)** (With [Lua modules doc](_init/doc/lua.md))

You can also run the `ask-aipack` agent. 

```sh
# IMPORTANT - Make sure you have the `OPENAI_API_KEY` or the key of your model in your environment
aipack run ask-aipack

# and then open the `.aipack/tmp/ask-aipack.md`
```

## Future Plans

- More Lua functions
- Agent module `my-module` may run `my-module/main.aip`, and running `my-module/some-other` will run `my-module/some-other.aip`
- Support Lua `Require`
- Full TUI/Ratatui 
- Split runtime to [agentic](https://crates.io/crates/agentic)
