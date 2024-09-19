# Data

```rhai
let file = file::load("./src/main.rs");

// return the file (with .name, .path, .content)
return #{ file: file };
```

# Instruction

Correct english in the comments of the rust content below, while preserving everything else as is. 

```rust
{{data.file.content}}
```

# Output

```rhai
let rust_code = md::extract_blocks(ai_output,"rust");
// file_save(data.file.path, rust_code);
return data.file.path
```