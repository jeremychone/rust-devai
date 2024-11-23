# APIs Summary

```lua

-- === utils.file

-- Load file text content in `file.content`
local file = utils.file.load("doc/some-file.md");
-- Save file content (will mkdirs)
utils.file.save("doc/some-file.md", "some new content");
-- List files matching a glob pattern
local all_doc_files = utils.file.list("doc/**/*.md");
-- Get the first file reference matching a glob pattern
local first_doc_file = utils.file.first("doc/**/*.md");
-- Load markdown sections from a file
local all_summary_section = utils.file.load("doc/readme.md", "# Summary");

-- === utils.path

-- Check if a path exists
local exists = path.exists("doc/some-file.md");
-- Check if a path is a file
local is_file = path.is_file("doc/some-file.md");
-- Check if a path is a directory
local is_dir = path.is_dir("doc/");
-- Get the parent directory of a path
local parent_dir = path.parent("doc/some-file.md");

-- === utils.git

-- Restore a file to its last committed state
utils.git.restore("src/main.rs");

-- === utils.web

-- Fetch content from a URL
local content = utils.web.get("https://example.com");

-- === utils.json

-- Parse a JSON string into a table
local obj = utils.json.parse('{"name": "John", "age": 30}');
-- Stringify a table into a JSON string
local json_str = utils.json.stringify(obj);
-- Stringify a table into a single line JSON string
local json_line_str = utils.json.stringify_to_line(obj);

-- === utils.rust

-- Prune Rust code to keep only function declarations
local result = utils.rust.prune_to_declarations("fn add(a: i32, b: i32) -> i32 { a + b }");

-- === utils.html

-- Prune HTML content to keep only visible content
local cleaned_html = utils.html.prune_to_content(html_content);

-- === utils.text

-- Replace markers in content with new sections
local updated_content = utils.text.replace_markers(content, new_sections);
-- Truncate content to a maximum length
local truncated_content = utils.text.truncate(content, 100, "...");
-- Ensure content ends with a single newline
local normalized_content = utils.text.ensure_single_ending_newline(content);
-- Remove the first line from content
local content_without_first_line = utils.text.remove_first_line(content);
-- Remove the first n lines from content
local content_without_first_n_lines = utils.text.remove_first_lines(content, n);
-- Remove the last line from content
local content_without_last_line = utils.text.remove_last_line(content);
-- Remove the last n lines from content
local content_without_last_n_lines = utils.text.remove_last_lines(content, n);
-- Decode HTML entities in content if needed
local decoded_content = utils.text.escape_decode_if_needed(content);
-- Decode HTML entities in content
local decoded_content = utils.text.escape_decode(content);

-- === utils.cmd

-- Execute a system command
local result = utils.cmd.exec("echo", "hello world");

-- === utils.devai

-- Return a before all response structure
local before_all_response = devai.before_all_response({
    before_all = "Some before all data",
    inputs = {"one", "two", "three", 4, "five"}
});
-- Skip input cycle with a reason
local skip_response = devai.skip("File already contains the documentation");

```

## CTX 

All of the lua script get the `CTX` table in scope to get the path of the runtime and agent. 

| Key                  | Value                                                     |
|----------------------|-----------------------------------------------------------|
| CTX.DEVAI_PARENT_DIR | `/absolute/path/to/devai_parent_dir`                      |
| CTX.DEVAI_DIR        | `./.devai`                                                |
| CTX.AGENT_NAME       | `command-ctx-reflect`                                     |
| CTX.AGENT_FILE_PATH  | `./.devai/custom/command-agent/command-ctx-reflect.devai` |
| CTX.AGENT_FILE_DIR   | `./.devai/custom/command-agent`                           |
| CTX.AGENT_FILE_NAME  | `command-ctx-reflect.devai`                               |
| CTX.AGENT_FILE_STEM  | `command-ctx-reflect`                                     |

- All path are relative `DEVAI_PARENT_DIR`
- The `AGENT_NAME` is the name given that resolve to the `AGENT_FILE_PATH`. Can use this name to do a `devai::run(CTX.AGENT_NAME, [])`
- Those are available in `devai run ..` and well as `devai solo ...`


## utils.file

### Functions

- `utils.file.load(path: string) -> FileRecord`
  
  Load a File Record object with its content.

  ```lua
  -- FileRecord
  {
    path    = "doc/README.md",
    content = "... text content of the file ...",
    name    = "README.md",
    stem    = "README",
    ext     = "md",
  }
  ```

- `utils.file.save(path: string, content: string)`

  Save a File Content into a path.

- `utils.file.list(include_glob: string) -> Array<FileRef>`

  List a set of file references (no content) for a given glob.

  ```lua
  -- An array/table of FileRef
  {
    path    = "doc/README.md",
    name    = "README.md",
    stem    = "README",
    ext     = "md",
  }
  ```

- `utils.file.first(include_glob: string) -> FileRef | nil`

  Return the first FileRef or Nil.

  ```lua
  -- FileRef or Nil
  {
    path    = "doc/README.md",
    name    = "README.md",
    stem    = "README",
    ext     = "md",
  }
  ```

- `utils.file.load_md_sections(path: string, headings: Value) -> Array<MdSection>`

  Load markdown sections from a file.

  ```lua
  -- Array/Table of MdSection
  {
    content = "Content of the section",
    heading = {content = "# Summary", level = 1, name = "Summary"},
  }
  ```

## utils.path

### Functions

- `path.exists(path: string) -> bool`

  Checks if the specified path exists.

- `path.is_file(path: string) -> bool`

  Checks if the specified path is a file.

- `path.is_dir(path: string) -> bool`

  Checks if the specified path is a directory.

- `path.parent(path: string) -> string | nil`

  Returns the parent directory of the specified path, or nil if there is no parent.

## utils.git

### Functions

- `utils.git.restore(path: string)`

  Will do a `git restore path`.

## utils.web

### Functions

- `utils.web.get(url: string) -> string`

  Fetch content from a URL.

## utils.json

### Functions

- `utils.json.parse(content: string) -> table`

  Parse a JSON string into a table.

  ```lua
  -- Returns a table representing the parsed JSON.
  ```

- `utils.json.stringify(content: table) -> string`

  Stringify a table into a JSON string.

  ```lua
  -- Returns a formatted JSON string.
  ```

- `utils.json.stringify_to_line(content: table) -> string`

  Stringify a table into a single line JSON string.

  ```lua
  -- Returns a single line JSON string.
  ```

## utils.rust

### Functions

- `utils.rust.prune_to_declarations(code: string) -> string`

  Trims Rust code to keep only function declarations.

## utils.html

### Functions

- `utils.html.prune_to_content(html_content: string) -> string`

  Strips non-content elements from the provided HTML content.

## utils.text

### Functions

- `utils.text.replace_markers(content: string, new_sections: array) -> string`

  Replaces markers in `content` with corresponding sections from `new_sections`.

- `utils.text.truncate(content: string, max_len: int, ellipsis?: string) -> string`

  Returns `content` truncated to a maximum length of `max_len`.

- `utils.text.ensure_single_ending_newline(content: string) -> string`

  Ensures that `content` ends with a single newline character.

- `utils.text.remove_first_line(content: string) -> string`

  Returns `content` with the first line removed.

- `utils.text.remove_first_lines(content: string, n: int) -> string`

  Returns `content` with the first `n` lines removed.

- `utils.text.remove_last_line(content: string) -> string`

  Returns `content` with the last line removed.

- `utils.text.remove_last_lines(content: string, n: int) -> string`

  Returns `content` with the last `n` lines removed.

- `utils.text.escape_decode_if_needed(content: string) -> string`

  Only escape if needed. Right now, the test only tests `&lt;`.

- `utils.text.escape_decode(content: string) -> string`

  Some LLMs HTML-encode their responses. This function returns `content`, HTML-decoded.

## utils.cmd

### Functions

- `utils.cmd.exec(cmd_name: string, args?: string | table) -> {stdout: string, stderr: string, exit: number}`

  Execute a system command with optional arguments.

## utils.devai

### Functions

- `devai.before_all_response(data: Value) -> Value`

  Can be returned in the `# Before All` Lua section to override the inputs.

  ```lua
  -- "_devai_": {
  --     "kind": "BeforeAllResponse",
  --     "data": {
  --         "inputs": ["one", "two", "three", 4, "five"],
  --         "before_all": "Some before all data"
  --     }
  -- }
  ```

- `devai.skip(reason: string) -> Value`

  Can be returned in `# Before All`, `# Data`, `# Output` Lua sections to skip this input or inputs cycle.

  ```lua
  -- "_devai_": {
  --     "kind": "Skip",
  --     "data": {
  --         "reason": "Some optional reason",
  --     }
  -- }
  ```