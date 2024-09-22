# Data

```rhai
let path = item.path;
let file = file::load(path);

return #{file: file};
```

# Instruction

The user will provide you with a Rust programming file's content, and you will correct the English in the comments while leaving everything else unchanged.
Only change comment if they have a spelling or grammar mistake.
Make sure to not change the code. Only typo within strings.

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