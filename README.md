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

ONE **Command Agent Markdown File** that defines the full agent flow:
- `items` get expanded from the `-f` file matches (more ways to generate items later).
- `-> Data` **scripting** for gaining full control over what data to put in the context.
- `-> Instruction` templating (Handlebars) to have full control over the prompt layout.
- `-> Output` **scripting** to gain full control over how to manage the AI output.

`Data`, `Instruction`, `Output` (and more later) are all defined in a single file (see below), which is called the **Command Agent File**.

Supports all models/providers supported by the [genai crate](https://crates.io/crates/genai) (see below for more information).

You can customize the model and concurrency in `.devai/config.toml`.

**v0.1.1 Note:** New `.devai/` file structure with the new `.devai` file extension. See [.devai/ folder structure](#devai-folder-structure).

**IMPORTANT**: In VSCode or your editor, map the `*.devai` extension to `markdown` to benefit from markdown highlighting. Devai agent files are markdown files.

**IMPORTANT**: Make sure to run this command line when everything is committed so that overwritten files can be reverted easily.

_P.S. If possible, please refrain from publishing `devai-custom` type crates on crates.io, as this might be more confusing than helpful. However feel free to fork and code as you wish._

## API Keys

**devai** uses the [genai crate](https://crates.io/crates/genai), and therefore the simplest way to provide the API keys for each provider is via environment variables in the terminal when running devai.

Here are the environment variable names used:

```
OPENAI_API_KEY
ANTHROPIC_API_KEY
MODEL_GEMINI
GEMINI_API_KEY
GROQ_API_KEY
COHERE_API_KEY
```

## Usage & Concept

Usage: `devai run proof-rust-comments -f "./src/main.rs"`

(or have any glob like `-f "./src/**/*.rs"`)
- This will initialize the `.devai/defaults` folder with the "Command Agent Markdown" `proof-rust-comments.md` (see [.devai/defaults/proof-comments.md`](./_init/agents/proof-comments.devai)) and run it with genai as follows: 
    - `-f "./src/**/*.rs"`: The `-f` command line argument takes a glob and will create an "item" for each file, which can then be accessed in the `# Data` scripting section.
    - `# Data`, which contains a ```rhai``` block that will get executed with the `item` value (the file reference in our example above).
        - With `rhai`, there are some utility functions to list files, load file content, and such that can then be used in the instruction section. 
    - `# Instruction`, which is a Handlebars template section, has access to `item` as well as the output of the `# Data` section, accessible as the `data` variable. 
        - This will be sent to the AI.
    - `# Output`, which now executes another ```rhai``` block, using the `item`, `data`, and `ai_output`, which is the string returned by the AI. 
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
# any file matching `./**/mod.rs` (those will become 'items' in the data section)
devai run proof-rust-comments -f mod.rs

# Verbose mode will print in the console what is sent to the AI, the AI response, and the output return if string-like
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
    - First argument is the command name. 
    - `-f` the file name or glob input files as items. Can have multiple `-f`
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
items_concurrency = 1 
```

## Future Plan

- Support for the `# Items` section with `yaml` or `Rhai`.
- More `Rhai` modules/functions.
- Support for `# Before All`, `# Before`, `# After`, and `# After All` (all `Rhai`).
- `--capture` will perform the normal run but capture the request and response in the request/response file.
