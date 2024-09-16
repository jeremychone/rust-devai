# Data

```rhai
let file = file_load(on_file_ref.path);

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
let rust_code = md_extract_first_rust(ai_output);
file_save(data.file.path, rust_code);

return data.file.path
```