# Data

```rhai
let path = item.path;
let file = file_load(path);

return #{file: file};
```

# Instruction

The user will provide you with a Rust programming file's content, and you will correct the English in the comments while leaving everything else unchanged.
Only change comment if they have a spelling or grammar mistake.

```rust
{{data.file.content}}
```

# Output

```rhai
let rust_code = md_extract_first_rust(ai_output);
file_save(data.file.path, rust_code);

let message = "File processed: " + data.file.path;
return message
```