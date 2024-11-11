<div align="center">

<a href="https://crates.io/crates/devai"><img src="https://img.shields.io/crates/v/devai.svg" /></a>
<a href="https://github.com/jeremychone/rust-devai"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=DSuvkCHdD5I&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj"><img alt="Static Badge" src="https://img.shields.io/badge/YouTube_devai_Intro-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# **devai** - **Command Agent File Runner**

```sh
# Install
cargo install devai

# Init (optional; will be executed on each run as well)
devai init

# Will fix all code comments in all matching files
devai run proof-rust-comments -f "./src/m*.rs"

# How: It will run the installed Command Agent file ".devai/defaults/proof-rust-comments.md" on all source files matching "./src/m*.rs"

# IMPORTANT: Make sure everything is committed before usage.
```

**TOP COLLABORATOR** Big **thanks** to [Stephan Philipakis](https://github.com/sphilipakis), a top **devai** collaborator contributing to the next-generation methodology for production coding with GenAI.

## **ONE Markdown** == **ONE Agent** 

At one of its simplest forms, there are 3 main stages

```sh
# Run the ./my-first-agent.devai on all files ending matching "**/*.md"
devai run ./my-first-agent.devai -f "**/*.md"
```

`./my-first-agent.devai`
``````md
# Data 

```rhai
// Here, there will be some script to run before each instruction
// Get access to `input` and can fetch more data and return it for the next stage
// Input is what was given by the command line (when -f, this will be the file metadata)

return #{
    file: file::load(input.path)
}
```

# Instruction

This is a handlebars section where we can include the data generated above. For example, 

Here is the content of the file to proofread

```{{input.ext}}
{{data.file.content}}
```

- Correct the English of the content above. 
- Do not modify it if it is grammatically correct. 

# Output

```rhai

// Here, we can take the ai_response.content, and do some final processing. 
// For example: 
// Here, we remove the eventual top markdown block. In certain cases, this might be useful. 
let content = md::outer_block_content_or_raw(ai_response.content);
// For code, it is nice to make sure it ends with one and only one new line. 
let content = text::ensure_single_ending_newline(content)
// More processing....

// input.path is the same as data.file.path, so we can use either
file::save(input.path, content )

```

``````

See [Complete Stages Description](#complete-stages-description) for more details

## Info

Supports all models/providers supported by the [genai crate](https://crates.io/crates/genai) (see below for more information).

You can customize the model and concurrency in `.devai/config.toml`.

**v0.1.1 Note:** New `.devai/` file structure with the new `.devai` file extension. See [.devai/ folder structure](#devai-folder-structure).

**IMPORTANT**: In VSCode or your editor, map the `*.devai` extension to `markdown` to benefit from markdown highlighting. Devai agent files are markdown files.

**IMPORTANT**: Make sure to run this command line when everything is committed so that overwritten files can be reverted easily.

_P.S. If possible, please refrain from publishing `devai-custom` type crates on crates.io, as this might be more confusing than helpful. However, feel free to fork and code as you wish._

### API Keys

**devai** uses the [genai crate](https://crates.io/crates/genai), and therefore, the simplest way to provide the API keys for each provider is via environment variables in the terminal when running devai.

Here are the environment variable names used:

```
OPENAI_API_KEY
ANTHROPIC_API_KEY
MODEL_GEMINI
GEMINI_API_KEY
GROQ_API_KEY
COHERE_API_KEY
```

On Mac, this CLI uses the Mac keychain to store the key value if it is not available in the environment variable. This will be extended to other OSes as it becomes more robust. 

## Complete Stages Description

In fact, there are 2 more stages to fully control the agent flow, and that is the `Before All` and `After All` stages. 

Here is a full description of the complete flow

- First, an agent receives zero or more inputs. 
    - Inputs can be given through the command line via: 
        - `-i` or `--input` to specify one input (can add multiple `-i/--input`)
        - `-f some_glob` will create one input per file matched with input as `{path, name, step, ext}`
    - Then the following stages happen (all optional)
- **Stage 1**: `# Before All` (rhai block) (optional) 
    - The `rhai` block has the following scope
        - `inputs` which is all of the inputs
    - Then can return
        - Nothing
        - Some data that will be available as `before_all` in the next stages.
            - e.g., `return #{"some": "data"}`
        - Override or generate inputs via 
            `return devai::before_all_response(#{inputs: [1, 2, 3]})`
        - or both by passing `#{inputs: ..., before_all: ...}` to the `devai::before_all_response` argument. 
- **Stage 2**: `# Data` (rhai block) (optional)
    - The `rhai` block gets the following variable in scope: 
        - `input` from the command line and/or Before All section (or null if no input)
    - Can return some data that will be labeled `data` in the future stage and can be used in the next steps. 
- **Stage 3**: `# Instruction` (handlebars template)
    - The content of the instruction is rendered via Handlebars, which is a templating engine, with the following variables in scope
        - `input` from Stage 1 or command line
        - `data` from Stage 2 (or null if no stage 2 or Stage 2 returns void/null/nothing)
- **Stage 4**: `# Output` (rhai block) (optional)
    - The `rhai` block will get the following scope
        - `input` from Stage 1 or command line (or null if no input)
        - `data` from Stage 2 (or null if no Stage 2 or Stage 2 returns void/null/nothing)
        - `ai_response` (if instruction) with 
            - `.content` the text content of the response
            - `.model_name` the model name it was executed with
    - Can return some data, which will be put in the `output` scope for the following stages
- **Stage 5**: `After All` (rhai block) (optional)
    - The `rhai` block will get the following scope
        - `inputs` the list of inputs from Stage 1 or command line
        - `outputs` the list of outputs from Stage 4 or null for each input
        - Note: the `inputs` and `outputs` arrays are kept in sync, and `null` will be in the output if not found. 
    - Can return some data, which will be labeled `after_all` for the caller of this function. e.g. `devai::run(agent, inputs)`



## Usage & Concept

Usage: `devai run proof-rust-comments -f "./src/main.rs"`

(or have any glob like `-f "./src/**/*.rs"`)
- This will initialize the `.devai/defaults` folder with the "Command Agent Markdown" `proof-rust-comments.md` (see [.devai/defaults/proof-comments.md`](./_init/agents/proof-comments.devai)) and run it with genai as follows: 
    - `-f "./src/**/*.rs"`: The `-f` command line argument takes a glob and will create an "input" for each file, which can then be accessed in the `# Data` scripting section.
    - `# Data`, which contains a ```rhai``` block that will get executed with the `input` value (the file reference in our example above).
        - With `rhai`, there are some utility functions to list files, load file content, and such that can then be used in the instruction section. 
    - `# Instruction`, which is a Handlebars template section, has access to `input` as well as the output of the `# Data` section, accessible as the `data` variable. 
        - This will be sent to the AI.
    - `# Output`, which now executes another ```rhai``` block, using the `input`, `data`, and `ai_response` (with `ai_response.content`), which is the string returned by the AI. 
        - It can save files in place or create new files. 
        - Later, it will even be able to queue new devai work.
- By default, this will run with `gpt-4o-mini` and look for the `OPENAI_API_KEY` environment variable.
- It supports all AI providers supported by the [genai crate](https://crates.io/crates/genai).
    - Here are the environment variable names per provider: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `COHERE_API_KEY`, `GEMINI_API_KEY`, `GROQ_API_KEY`.
    - On Mac, if the environment variable is not present, it will attempt to prompt and get/save it from the keychain, under the devai group.

## `devai` command arguments

```sh
# Will create/update the .devai/ settings folder (not required, automatically runs on "run")
devai init

# Will execute the proof-rust-comments.md from `.devai/customs/` or `.devai/defaults/` on 
# any file matching `./**/mod.rs` (those will become 'inputs' in the data section)
devai run proof-rust-comments -f mod.rs

# Verbose mode will print in the console what is sent to the AI, the AI response, and the output returned if string-like
devai run proof-rust-comments -f mod.rs --verbose 

# Verbose and watch mode. Every time proof-rust-comments is updated, it will run it again
devai run proof-rust-comments -f main.rs -v -w

# Will perform the verbose, watch, but in dry mode request, will print only the rendered instruction
devai run proof-rust-comments -f main.rs -v -w --dry req

# Will perform the verbose, watch, but in dry mode response, will print rendered instruction, AI response
# and will NOT execute the data
devai run proof-rust-comments -f main.rs -v -w --dry res

# Happy coding!
```

- `init` sub-command - initialize or update the `.devai/` folder (non-destructive, only adds files that are missing)
- `run` sub-command
    - The first argument is the command name. 
    - `-f` the file name or glob input files as inputs. Can have multiple `-f`
    - `--verbose` (`-v`) will print the rendered output in the command line.
    - `--dry req` will perform a dry run of the request by just running the **data** and **instruction** sections. Use `--verbose` to print out the sections.
    - `--dry res` will perform a dry run of the request, send it to the AI, and return the AI output (does not return data). Use `--verbose` to see what has been sent and returned.

## devai folder structure

(Updated in version `0.1.1` - migration from `0.1.0` implemented on `devai run` and `devai init`)

- `.devai/` - The root folder of devai
    - `custom/` - Where user custom agents and templates are stored. These will take precedence over the `.devai/default/...` matching files.
        - `command-agent/` - The custom agents. 
        - `new-template/` - Template(s) used to create new agents, e.g., `devai new my-new-cool-agent`
            - `command-agent/` - The folder containing the custom templates for command agents.
            - `solo-agent/` - The folder containing custom templates for solo agents (coming later)
    - `default/` - The default command agents and templates provided by devai (these files will only be created if missing)
        - `command-agent/` - The default command agents.
        - `new-template/` - The default template(s) used to create new agents, e.g., `devai new my-new-cool-agent`
            - `command-agent/` - The folder containing the default templates for command agents.
            - `solo-agent/` - The folder containing the default templates for solo agents (coming later)

## Example of a Command Agent File

`.devai/defaults/proof-rust-comments.md` (see [.devai/defaults/proof-rust-comments.md`](./_base/agents/proof-rust-comments.md))

## Config

On `devai run` or `devai init`, a `.devai/config.toml` will be created with the following:

```toml
[genai]
# Required (any model rust genai crate supports).
model = "gpt-4o-mini" 

[runtime]
# Default to 1 if absent. A great way to increase speed when using remote AI services.
inputs_concurrency = 1 
```

## Future Plan

- Support for the `# inputs` section with `yaml` or `Rhai`.
- More `Rhai` modules/functions.
- Support for `# Before All`, `# Before`, `# After`, and `# After All` (all `Rhai`).
- `--capture` will perform the normal run but capture the request and response in the request/response file.
