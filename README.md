# **devai** - **Command Agent File Runner**

Gain full control over how to apply a reusable **Command Agent** on multiple files at once.

ONE **Command Agent Markdown File** that defines the full agent flow:
- `items` to simply run the same **Command Agent** on multiple files at once.
- `-> Data` **scripting** for getting full control over what data to put in the context.
- `-> Instruction` templating (Handlebars) to have full control over the prompt layout.
- `-> Output` **scripting** to get full control over how to manage the AI output.

**IMPORTANT**: Make sure to run this command line when everything is committed, so that overwritten files can be reverted easily.

STILL IN HEAVY DEVELOPMENT... But it's starting to get pretty cool.

_P.S. If possible, try to refrain from publishing `devai-custom` type crates, as this might be more confusing than helpful. However, any other name is great._


## Usage & Concept

**IMPORTANT**: Make sure all is commited before usage. 

Usage: `devai run proof-comments -f "./src/main.rs"`

(or have any glob like `-f "./src/**/*.rs"` )

- This will initialize the `.devai/defaults` folder with the "Command Agent Markdown" `proof-comments.md` (see [.devai/defaults/proof-comments.md`](./_base/agents/proof-comments.md)) and run it with genai as follows: 
    - `# Data`, which contains a ```rhai``` block that will get executed with the `item` value (the file reference in our example above).
        - With `rhai`, there are some utility functions to list files, load file content, and such that can then be used in the instruction section. 
    - `# Instruction`, which is a Handlebars template section, has access to `item` as well as the output of the `# Data` section, accessible as the `data` variable. 
        - This will be sent to the AI.
    - `# Output`, which now executes another ```rhai``` block, using the `item`, `data`, and `ai_output`, which is the string returned by the AI. 
        - It can save files in place or create new files. 
        - Later, it will even be able to queue new devai work.


## Example of a Agent Command File

`.devai/defaults/proof-comments.md` (see [.devai/defaults/proof-comments.md`](./_base/agents/proof-comments.md))
``````md
# Data

```rhai
let path = item.path;
let file = file::load(path);

return #{file: file};
```

# Instruction

The user will provide you with the content of a Rust programming file, and you will correct the English in the comments while leaving everything else unchanged.

Very important: 
- Only change comments if they have spelling or grammar mistakes.
- Make sure to NOT change the code. Only correct typos within strings.
- Do not change the whitespace like tabs or spaces.

```rust
{{data.file.content}}
```

# Output

```rhai
let rust_blocks = md::extract_blocks(ai_output, "rust");
let first_rust_block = rust_blocks[0];
let rust_code = text::escape_decode_if_needed(first_rust_block);

file::save(data.file.path, rust_code);

let message = "File processed: " + data.file.path;
return message;
```

``````

## Config

On `devai run` or `devai init` a `.devai/config.toml` will be created with the following:

```toml
[genai]
# Required (any model rust genai crate support).
model = "gpt-4o-mini" 

[runtime]
# Default to 1 if absent. Great way to increase speed when remote AI services.
items_concurrency = 1 
```


## Future Plan

- Support for the `# Items` section with `yaml` or `Rhai`.
- Support for `# Config` to override some `config.toml` properties (e.g., model).
- More `Rhai` modules/functions.
- Support for `# Before All`, `# Before`, `# After`, and `# After All` (all `Rhai`).
- `--dry-req` will perform a dry run of the request by just saving the content of the request in a file.
- `--dry-res` will perform a real AI request but just capture the AI response in a file (the request will be captured as well).
- `--capture` will perform the normal run but capture the request and response in the request/response file.

## Future Command Agent File

