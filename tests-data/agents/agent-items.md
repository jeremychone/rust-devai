# Items

```yaml

- name: Jean Dumond
  title: Director Engineering
- name: Mike Donavan
  title: Director Markeing

```

# Data

```rhai
let file = file_load(item.path);

// return the file (with .name, .path, .content)
return #{ file: file };
```

# Instruction

Correct english in the comments of the rust content below, while preserving everything else as is. 

{{item.name}}

```rust
{{data.file.content}}
```

# Output

```rhai
let rust_code = md::extract_blocks(ai_output, "rust")[0];
file_save(data.file.path, rust_code);

return data.file.path
```