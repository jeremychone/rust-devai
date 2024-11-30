<div align="center">

<a href="https://crates.io/crates/devai"><img src="https://img.shields.io/crates/v/devai.svg" /></a>
<a href="https://github.com/jeremychone/rust-devai"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=DSuvkCHdD5I&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj"><img alt="Static Badge" src="https://img.shields.io/badge/YouTube_devai_Intro-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# **devai** - **Command Agent File Runner**

```sh
# Install (for now, the simpler way to install is with cargo install)
cargo install devai

# Init (optional; will be executed on each run as well)
devai init

# Will proofread and update the direct .md file from the current directory
devai run proof-read -f "./*.md"
# Can use multiple globs or direct file -f "./*.md" -f "./doc/**/*.md"

# How it works: 
#   It will run the installed Command Agent file ".devai/defaults/proof-read.devai" on all source files matching "./src/m*.rs"

# IMPORTANT: Make sure everything is committed before usage.
```

# Key Concept - **One Agent** == **One Markdown** 

The main **devai** concept is to minimize the friction of creating and running an agent while giving maximum control over how we want those agents to run, and maximizing iteration speed to mature them quickly.

Here are some of the key **devai** charactheristic. 

- **One Agent** == **One Markdown** 
    -(i.e., `my-agent.devai`, a `.devai` is a markdown file with multi-stage sections desribed below)
- **Multi AI Provider / Models** Supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come. 
- **Lua** for Scripting.
- **Handlebars** for prompt templating. 
- **Multi Stage** process, where ALL steps are optionals.
    - `# Before All` - (Lua) - reshape / generate inputs, and add command global data to sope (the "map" of the map/reduce capability). 
    - `# Data` - (Lua) - Gather additional data per input, and return it for next stages. 
    - `# System`, `# Instructions` - (Handlebars) - Customize prompt with the `data` and `before_all` data. 
        - Can event have `# Assistant` for "Jedi Mind Trick"
    - `# Output` - (Lua) Will process the `ai_response` from the LLM. Otherwise, `ai_response.content` will get outputed to terminal
    - `# After All` - If prefent, get call with `inputs` and `outputs` variables for some post processing after all of the input are completed. 

[more info on stages](_init/doc/README.md#complete-stages-description)

## [devai doc](_init/doc/README.md)

See the devai doc at **[_init/doc/README.md](_init/doc/README.md)**

You can also run the `ask-devai` agent. 

```sh
# IMPORTANT - Make sure you have the `OPENAI_API_KEY` or the key of your model in you env
devai run ask-devai

# and then the open the `.devai/tmp/ask-devai.md`
```

[more info on api keys](_init/doc/README.md#api-keys)

## Thanks

**TOP COLLABORATOR** Big **thanks** to [Stephan Philipakis](https://github.com/sphilipakis), a top **devai** collaborator contributing to the next-generation methodology for production coding with GenAI.


## Future Plan

- More lua functions
- Agent module `my-module` may run `my-module/main.devai` and running `my-module/some-other` will run `my-module/some-other.devai`
- Support Lua `Require`
- Full TUI/Ratatui 
- Split runtime to [agentic](https://crates.io/crates/agentic)