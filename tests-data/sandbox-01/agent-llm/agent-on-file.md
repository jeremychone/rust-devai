# Data

```rhai
let file = file::load(input.path);

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
let rust_code = md::extract_blocks(ai_output, "rust")[0];
// file::save(data.file.path, rust_code); // do not save, otherwise, cwt run forever

return #{data_path: data.file.path, 
        input_name: input.name,
        ai_content: ai_output}
```