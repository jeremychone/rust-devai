# Data

```rhai
let path = item.path;
let file = file_load(path);

return #{file: file};
```

# Instruction

The user will provide you with a Rust programming file's content, and you will correct the English in the comments while leaving everything else unchanged.
Only change comment if they have a spelling or grammar mistake.
Make sure to not change the code. Only typo within strings.
Do not escape any angle bracket and such.

```rust
{{data.file.content}}
```

# Output

```rhai
let rust_code = md_extract_first_rust(ai_output);
let rust_code = text_escape_decode(rust_code);
file_save(data.file.path, rust_code);

let message = "File processed: " + data.file.path;
return message
```