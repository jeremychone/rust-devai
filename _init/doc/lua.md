# APIs / Context Summary

**devai** injects the following modules/variables into the various script stages:

- In all scripts (`# Before All`, `# Data`, `# Output`, `# After All`)
  - [utils](#utils) - A set of utility functions and submodules.
  - [devai](#devai) - A module to control the devai flow (e.g., `return devai.skip("No need to perform this input")`)
  - [CTX](#ctx) - A set of constants mostly related to the various paths used for this execution (e.g., `CTX.AGENT_FILE_PATH`)
<br/>

- In the `# Before All` stage
  - `inputs` - The list of inputs given to the run command (in solo mode, only one)
    - When `-f "**/some/glob*.*"` is used, each input will be the matching `FileMeta` object.
<br/>

- In the `# Data` stage
  - `input` - The individual input given from the devai run
    - In the case of `-f ...`, it will be the [FileMeta](#filemeta) structure for each file.
<br/>

- In the `# Output` stage
  - `data` - Whatever is returned by the `# Data` script.
  - `ai_response` - The [AiResponse](#airesponse)
<br/>

- In the `# After All` stage
  - `inputs` - The inputs sent or modified by `# Before All`
  - `outputs` - The outputs returned by the `# Output` stage
    - The same order as `inputs`, and `nil` when an item has been skipped or the output did not return anything.

## utils

The utils top module is comprised of the following submodules.

### utils.file

See [FileRecord](#filerecord), [FileMeta](#filemeta), [MdSection](#mdsection) for return types.

All relative paths are relative to the workspace dir, which is the parent dir of the `.devai/` folder. 

```lua
-- Load file text content and return its FileRecord (See below), with `.content`
local file = utils.file.load("doc/some-file.md")                -- FileRecord

-- Save file content (will create directories as needed)
utils.file.save("doc/some-file.md", "some new content")         -- void (no return for now)

-- Append content to file (create file and directoris as needed) 
utils.file.append("doc/some-file.md", "some new content")       -- void (no return for now)

-- List files matching a glob pattern
--   (file.path will be relative to devai workspace dir)
local all_doc_files = utils.file.list("doc/**/*.md")            -- {FileMeta, ...}

-- List files matching a glob pattern and options (for now only base_dir)
--   (file.path will be relative to base dir, which is relative to workspace dir)
local all_doc_files = utils.file.list("**/*.md", {base_dir: "doc/})            -- {FileMeta, ...}

-- List files and load their content (or with the options as well)
local all_files = utils.file.list_load({"doc/**/*.md", "src/**/*.rs"})           -- {FileRecord, ...}

-- Get the first file reference matching a glob pattern (or with options as well)
local first_doc_file = utils.file.first("doc/**/*.md")          -- FileMeta | Nil

-- Ensure a file exists by creating it if not found
local file_meta = utils.file.ensure_exists("./some/file.md", "optional content") 
                                                                -- FileMeta

-- Load markdown sections from a file
-- If the second argument is absent, then all sections will be returned (nested as items as well)
local sections = utils.file.load_md_sections("doc/readme.md", "# Summary")  
                                                                 -- {MdSection, ...}
```

### utils.path

All relative paths are relative to the workspace dir, which is the parent dir of the `.devai/` folder. 

```lua
-- Check if a path exists
local exists = utils.path.exists("doc/some-file.md")         -- bool
-- Check if a path is a file
local is_file = utils.path.is_file("doc/some-file.md")       -- bool
-- Check if a path is a directory
local is_dir = utils.path.is_dir("doc/")                     -- bool
-- Get the parent directory of a path
local parent_dir = utils.path.parent("doc/some-file.md")     -- string
-- Split for parent and filename
local parent_dir, file_name = utils.path.split("path/to/some-file.md") -- parent, file
-- returns "path/to", "some-file.md"
-- Join path
local path = utils.path.join("path", "to", "some-file.md")   -- string
-- "path/to/some-file.md"
```


## utils.text

```lua
local trimmed = utils.text.trim(content)        -- string
local trimmed = utils.text.trim_start(content)  -- string
local trimmed = utils.text.trim_end(content)    -- string

-- Truncate content to a maximum length
-- - ellipsis - optional third argument
local truncated_content = utils.text.truncate(content, 100, "...")        -- string

-- Ensure
-- - second argument of type `{prefix = string, suffix = string}` both optional
-- - if defined, it will add the prefix and suffix if they are not present
utils.text.ensure(content, {prefix = "./", suffix = ".md"}) -> string

-- Ensure content ends with a single newline
local normalized_content = utils.text.ensure_single_ending_newline(content)
                                                                           -- string

-- split_first - Split the first occurrence of a separator
local content = "some first content\n===\nsecond content"
local first, second = utils.text.split_first(content,"===")
-- first  = "some first content\n"
-- second = "\nsecond content"
-- NOTE: When no match, second is nil. 
--       If matched, but nothing after, second is ""

-- Remove the first line from content
local content_without_first_line = utils.text.remove_first_line(content)  -- string
-- Remove the last line from content
local content_without_last_line = utils.text.remove_last_line(content)    -- string

-- (Advanced) Replace markers in content with new sections
--   - Markers for now are in between `<<START>>` and `<<END>>`
local updated_content = utils.text.replace_markers(content, new_sections) -- string
```


### utils.md

See [MdBlock](#mdblock)

```lua
-- Extract all blocks
utils.md.extract_blocks(md_content: string) -> Vec<MdBlock>
-- Extract blocks for the language 'lang'
utils.md.extract_blocks(md_content: string, lang: string) -> Vec<MdBlock>
-- Extract blocks (with or without language, and extrude: content, which the remaining content)
utils.md.extract_blocks(md_content: String, {lang: string, extrude: "content"})
-- Extract, parse, and merge the `#!meta`, and return the value and the concatenated remaining text.
local meta, remain = utils.md.extract_emai(md_content: String, {lang: string, extrude: "content"})

-- If content starts with ```, it will remove the first and last ```, and return the content in between
-- Otherwise, it returns the original content
local content = utils.md.outer_block_content_or_raw(content) -- string

```

### utils.json

```lua
-- Parse a JSON string into a table
local obj = utils.json.parse('{"name": "John", "age": 30}')  -- Object (lua table)
-- Stringify a table into a JSON string
local json_str = utils.json.stringify(obj)                   -- string
-- Stringify a table into a single-line JSON string
local json_line_str = utils.json.stringify_to_line(obj)      -- string
```

### utils.lua

```lua
-- Return a pretty string of a lua value
local dump = utils.lua.dump(some_var)  -- string
print(dump)
```

### utils.rust

```lua
-- === utils.rust
-- Prune Rust code to keep only function declarations (removes function bodies)
local result = utils.rust.prune_to_declarations(code)  -- string
```

### utils.git

```lua
-- Restore a file to its last committed state
utils.git.restore("src/main.rs")                       -- void
```

### utils.web

See [WebResponse](#webresponse).

```lua
-- Fetch web_response from a URL (see WebResponse object)
local web_response = utils.web.get("https://example.com")   -- WebResponse 
-- Return an exception when a web request cannot be executed (e.g., bad URL, remote server not available)

-- Do a post 
local web_response = utils.web.post("https://httpbin.org/post", { some = "stuff"})
-- if data is table, will be serialize as json, and content_type `application/json` 
-- If data is string, then, just as is, and `plain/text`
```

#### WebResponse

The `WebResponse`

```lua
{
 success = true,
 status = number,
 url = string,
 content = string | table, 
}
-- .content will be Lua Table if response content_type is application/json
--          otherwie, just string
```


## utils.html

```lua
-- Prune HTML content to remove some empty tags, comments, and such
local cleaned_html = utils.html.prune_to_content(html_content)  -- string
```
## utils.cmd

See [CmdResponse](#cmdresponse)

```lua
-- Execute a system command utils.cmd.exec(cmd_name, cmd_args)
local result = utils.cmd.exec("ls", {"-ll", "./**/*.md"})  -- CmdResponse
```

## devai

devai also provides the `devai` module in the context of all scripts, which allows control over the devai flow.

```lua
-- Return a before all response structure
local before_all_response = devai.before_all_response({
    before_all = "Some before all data",
    inputs = {"one", "two", "three", 4, "five"}
})

-- Skip input cycle with an optional reason
-- This can be used in the `# Data`, `# Before All`, and `# Output` stages
local skip_response = devai.skip("File already contains the documentation")
```

## CTX

All Lua scripts get the `CTX` table in scope to get the path of the runtime and agent.

| Key                 | Value                                  |
|---------------------|----------------------------------------|
| CTX.WORKSPACE_DIR   | `/absolute/path/to/workspace_dir`      |
| CTX.DEVAI_DIR       | `./.devai`                             |
| CTX.AGENT_NAME      | `my-agent`                             |
| CTX.AGENT_FILE_PATH | `./.devai/custom/agent/my-agent.devai` |
| CTX.AGENT_FILE_DIR  | `./.devai/custom/agent`                |
| CTX.AGENT_FILE_NAME | `my-agent.devai`                       |
| CTX.AGENT_FILE_STEM | `my-agent`                             |

- All paths are relative to `WORKSPACE_DIR`
- The `AGENT_NAME` is the name provided that resolves to the `AGENT_FILE_PATH`. You can use this name to do a `devai::run(CTX.AGENT_NAME, [])`
- These are available in `devai run ..` as well as `devai solo ...`

# Common Types

## AIResponse

In the `# Output` section, the `ai_response` is injected in the scope with the following structure: 

```lua
-- ai_response in '# Output' lua section

ai_response: {
  content:            string | nil, -- Typically not null
  reasoning_content:  string | nil, -- If the model gives it back, e.g., deepseek-reasoner, deepseek still in ollama & Groq
  usage: {
    prompt_tokens:     number,
    completion_tokens: number,

    completion_tokens_details: { -- won't be nil
      accepted_prediction_tokens: number | nil,
      rejected_prediction_token:  number | nil,
      audio_token:                number | nil,
      reasoning_tokens:           number | nil,
    },
    
    prompt_tokens_details: {     -- won't be nil
      cached_tokens: number | nil,
      audio_tokens:  number | nil,
    }
  }
}
```

## FileMeta

The `FileMeta` data structure represents the information of a given file without its content.
This is what `-f "**/some/glob*.*"` would return for each of the inputs.

```lua
{
  path    = "doc/README.md",
  name    = "README.md",
  stem    = "README",
  ext     = "md",
}
```

## FileRecord

The `FileRecord` data structure represents the information of a given file plus its text content.
This is what is returned by `utils.file.load(path)`, for example.

```lua
{
  path    = "doc/README.md",
  name    = "README.md",
  stem    = "README",
  ext     = "md",
  content = "the content of the file",
}
```

## MdSection

The `MdSection` is a markdown section with the following representation:

```lua
{
  content = "_section_content",     -- after the eventual heading
  heading_content = "# Summary",    -- can be "" if there is no heading (top section)
  heading_name    = "Summary",      -- can be "" if there is no heading
  heading_level   = 1,              -- Will be 0 when there is no heading
  heading_raw     = "# Summary\n",  -- Will be "" when there is no heading. Simplifies reconstitution logic     
}
```

## MdBlock

The `MdBlock` is a markdown section with the following representation:

```lua
{
  content = "_block_content_",     -- The content of the block
  lang = "js",                     -- string | nil 
}
```

## CmdResponse

The `CmdResponse` is returned by `utils.cmd.exec`

```lua
{
  stdout = string,  -- Standard output from the command
  stderr = string,  -- Standard error from the command
  exit   = number   -- Exit code (0 for success)
}
```