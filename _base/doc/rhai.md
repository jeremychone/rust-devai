# RHAI Modules Documentation

This document provides an overview of the RHAI modules implemented in the project. Each module exposes specific functions that can be utilized within the RHAI scripting engine. Examples are provided for each function to demonstrate their usage and the structure of their return values.

## Module: file

The `file` module exposes functions used to read, write, or modify files.

### file::load(file_path: string) -> FileRecord

Reads the file specified by `file_path`, returning the contents of the file along with helpful metadata.

**Example:**

```rhai
let record = file::load("./path/to/file.js");
// record = {
//   path: "./path/to/file.js",
//   name: "file.js",
//   stem: "file",
//   ext: "js",
//   content: "// JavaScript file content..."
// }
```

### file::save(file_path: string, content: string)

Writes `content` to the specified `file_path`.

**Example:**

```rhai
file::save("./path/to/file.js", "console.log('Hello, World!');");
// After execution, the file "./path/to/file.js" contains:
// console.log('Hello, World!');
```

### file::list(glob: string) -> Vec<FileRef>

Expands `glob`, returning a list of all matching file paths along with helpful metadata.

**Example:**

```rhai
let files = file::list("src/**/*.rs");
// files = [
//   { path: "src/main.rs", name: "main.rs", stem: "main", ext: "rs" },
//   { path: "src/lib.rs", name: "lib.rs", stem: "lib", ext: "rs" }
// ]
```

## Module: md

The `md` module exposes functions that process markdown content. Useful for processing LLM responses.

### extract_blocks(md_content: string) -> Vec<MdBlock>

Parses the markdown provided by `md_content` and extracts each code block, returning a list of blocks.

**Example:**

```rhai
let blocks = md::extract_blocks("# Title\n```rust\nfn main() {}\n```\n```js\nconsole.log('Hello');\n```");
// blocks = [
//   { lang: "rust", content: "fn main() {}" },
//   { lang: "js", content: "console.log('Hello');" }
// ]
```

### md::extract_blocks(md_content: string, lang_name: string) -> Vec<MdBlock>

Parses the markdown provided by `md_content` and extracts each code block with a language identifier that matches `lang_name`.

**Example:**

```rhai
let rust_blocks = md::extract_blocks("# Title\n```rust\nfn main() {}\n```\n```js\nconsole.log('Hello');\n```", "rust");
// rust_blocks = [
//   { lang: "rust", content: "fn main() {}" }
// ]
```

### md::outer_block_content_or_raw(md_content: string) -> string

Attempts to extract the content from the first triple backticks until the last triple backticks. If no backticks are found, returns the raw content.

**Example:**

```rhai
let content = md::outer_block_content_or_raw("Some text\n```python\ndef hello(): pass\n```");
// content = "python\ndef hello(): pass"
```


## Module: text

The `text` module exposes functions that process text.

### text::escape_decode(content: string) -> string

Some LLMs HTML-encode their responses. This function returns `content`, HTML-decoded.

**Example:**

```rhai
let decoded = text::escape_decode("Hello &lt;World&gt;!");
// decoded = "Hello <World>!"
```

### text::escape_decode_if_needed(content: string) -> string

Only escapes if needed. Returns `content` after selectively decoding certain HTML tags (currently only decodes `&lt;`).

**Example:**

```rhai
let decoded1 = text::escape_decode_if_needed("No encoding here.");
// decoded1 = "No encoding here."

let decoded2 = text::escape_decode_if_needed("Hello &lt;World&gt;!");
// decoded2 = "Hello <World>!"
```

### text::remove_first_line(content: string) -> string

Returns `content` with the first line removed.

**Example:**

```rhai
let result = text::remove_first_line("Line1\nLine2\nLine3");
// result = "Line2\nLine3"
```

### text::remove_first_lines(content: string, n: int) -> string

Returns `content` with the first `n` lines removed.

**Example:**

```rhai
let result = text::remove_first_lines("Line1\nLine2\nLine3\nLine4", 2);
// result = "Line3\nLine4"
```

### text::remove_last_line(content: string) -> string

Returns `content` with the last line removed.

**Example:**

```rhai
let result = text::remove_last_line("Line1\nLine2\nLine3");
// result = "Line1\nLine2"
```

### text::remove_last_lines(content: string, n: int) -> string

Returns `content` with the last `n` lines removed.

**Example:**

```rhai
let result = text::remove_last_lines("Line1\nLine2\nLine3\nLine4", 2);
// result = "Line1\nLine2"
```


## Module: git

The `git` module exposes functions that call `git` commands.

### git::restore(file_path: string) -> string

Calls `git restore {file_path}` and returns the output (stdout) of that call.

**Example:**

```rhai
let output = git::restore("./path/to/file.js");
// output = "Restored ./path/to/file.js to HEAD."
```

*Note:* If `git restore` outputs to stderr, it will be published to the hub and an error will be returned.


## Module: devai

The `devai` module provides functions for interacting with the devai system, such as skipping actions.

### devai::action_skip() -> SkipActionDict

Used in the `# Data` section to return a devai skip action so that the item is not included in the next flow (instruction > AI > data).

**Example:**

```rhai
if item.name == "mod.rs" {
    return devai::action_skip();
}
// {
//   "_devai_": {
//     "action": "skip"
//   }
// }
```

### devai::action_skip(reason: string) -> SkipActionDict

Used in the `# Data` section to return a devai skip action with a reason so that the item is not included in the next flow. The reason will be printed.

**Example:**

```rhai
if item.name == "mod.rs" {
    return devai::action_skip("mod.rs does not need to be processed by this agent");
}
// {
//   "_devai_": {
//     "action": "skip",
//     "data": {
//       "reason": "mod.rs does not need to be processed by this agent"
//     }
//   }
// }
```

