# Data

```rhai
let path = item.path;
let file = file::load(path);

return #{file: file};
```

# Instruction

- The user will provide you with the content of a Rust programming file. 
- You will correct the English in the comments, but leave everything else unchanged. 
- Only modify comments if there is a spelling or grammar mistake. 
- Make sure not to change the code, except for typos within strings.
- Do not change the code itself, only comments.

```rust
{{data.file.content}}
```

# Output

```rhai
let rust_blocks = md::extract_blocks(ai_output, "rust");
let first_rust_block = rust_blocks[0].content;
let rust_code = text::escape_decode_if_needed(first_rust_block);

file::save(data.file.path, rust_code);

// This will be printed
return "File processed: " + data.file.path
```