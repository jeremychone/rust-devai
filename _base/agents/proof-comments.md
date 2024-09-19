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
let first_rust_block = blocks[0];
let rust_code = text::escape_decode_if_needed(rust_code);

file::save(data.file.path, rust_code);

let message = "File processed: " + data.file.path;
return message
```