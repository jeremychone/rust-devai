# Config

```toml
[genai]
model = "test_model_for_demo"

[runtime]
inputs_concurrency = 8
```

# Data

```rhai

// Some scripts that load the data
// Will have access to the command line args after command name

let src_builder_file = file::load("./src/client/builder.rs");
// src_builder_file.name: "builder.rs"
// src_builder_file.path: "./src/client/builder.rs"
// src_builder_file.content: ".... content of the file ...."

return "hello"
```

# Instruction

Some paragraph for instruction

```some
# block-01
```

```some
block-02
```

``````

```
# block-03
```

# Instruction
```````

- One 
- Two
* Three

1. four
2. five

- . some instruction, will support handlebars in this block (first system content) ..
* final stuff

# Output

```rhai
/// Optional output processing.
```
