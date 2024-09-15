**devai** - **Command Agent runner to accelerate production coding. File based, fully customizable, NOT for building snake games.**

STILL IN HEAVY DEVELOPMENT

Cherry-pick the code and make it work for yourself.

_P.S. If possible, try to refrain from publishing `devai-custom` type crates, as this might be more confusing than helpful. However, any other name is great._

Usage: `da _command_name_ ...`

It's very rudimentary at this stage and will change significantly between `0.0.z` releases.

**IMPORTANT** Make sure to use it on a fully committed repository so that if changes break things, you can quickly revert.

## Concept

This is very early experimentation, but the goal is to continue maturing the concept.

The idea is to have more "Command Agents" described as markdown configuration files that can be run with the devai command (CLI).

Right now, we only have `proof-comments.md`, and the command line logic is primitive and hardcoded. It runs for all files or specified files and saves the AI Rust result back to the file.

## Future Plan

- Incorporate `rhai` to allow scripting in the **Command Agent Files**.
- Support "structured" Markdown for the **Command Agent Files** to provide more customizability.
- `--dry-req` will do a dry run of the request by just saving the content of the request in a file.
- `--dry-res` will do a real AI request but just capture the AI response in a file (request will be captured as well).
- `--capture` will do the normal run but will capture the req and response in the req/response file.

## Future Command Agent File

### Simplest form, just an instruction.

Right now, this will run it for the source file targeted, and whatever the AI returns will get saved back to this file. (very primitive, and just a proof of concept for now)

```md
... some instruction ...
```

### Advanced

``````md
## Data

```rhai
// Some scripts that load the data
// Will have access to the command line args after command name

let src_builder_file = read_file("./src/client/builder.rs");
// src_builder_file.name: "builder.rs"
// src_builder_file.path: "./src/client/builder.rs"
// src_builder_file.content: ".... content of the file ...."
```

## Instruction

... some instruction, will support handlebars in this block (first system content) ..

## message: System

... possible extra system message (will support handlebars) ...

## message: User

... possible extra user message (will support handlebars) ...

## Output Processing

```rhai
/// optional output processing.
```

`````