`.` minor | `-` Fix | `+` Addition | `^` improvement | `!` Change | `*` important | `>` Refactor

## 2025-02-28 - [v0.6.3](https://github.com/jeremychone/rust-devai/compare/v0.6.2...v0.6.3)

- `+` **pricing** - first pass at adding pricing. Now, when available, `ai_response.price_usd` and added in `ai_response.info`
- `+` **install** - Now can do `aip install path/to/file.aipack`
- `>` major internal refactor - pack, packer (and first wire for aip install)

## 2025-02-28 - [v0.6.2](https://github.com/jeremychone/rust-devai/compare/v0.6.1...v0.6.2)

- `-` @coder - normalize coder to use four backtics for code block
- `-` jc@coder - fix the 6 backticks to be 4, which is the correct standard (for extract_blocks and extract_sections)
- `+` pack - template generation
- `+` pack - first pass at pack dir `aip pack some/path/to/dir

## 2025-02-27 - [v0.6.1](https://github.com/jeremychone/rust-devai/compare/v0.6.0...v0.6.1)

- `!` workspace - do not add .aipack/pack/custom on init anymore (still part of pack resolution)
- `-` aipbase - fix core/ask-aipack/

## 2025-02-26 - **v0.6.0** **BIG CHANGE - now AIPACK**

**BIG CHANGE - now AIPACK with agent packs `aip run namespace@pack_name`**

- **same codebase**, **same feature set**, **same licenses (MIT or Apache)**

- But now **ai packs centric**, which is going to bring huge value for the users and community.

- See [README.md](README.md)

## 2025-02-23 - [v0.5.12](https://github.com/jeremychone/rust-devai/compare/v0.5.11...v0.5.12)

- `*` readme - NOTICE about AIPACK migration
- `.` rust - update to 2024 edition, rust-version 1.85
- `^` lua - utils.text.extract_line_blocks error handling when options.starts_with is missing
- `^` agent - coder - fine tune prompt & move the initial doc below the ====

## 2025-02-22 - [v0.5.11](https://github.com/jeremychone/rust-devai/compare/v0.5.9...v0.5.11)

- `+` Parametric Agents with support for `#!meta` prompt code blocks
- `+` `coder` agent
- ... many more

## 2025-01-27 - [v0.5.9](https://github.com/jeremychone/rust-devai/compare/v0.5.8...v0.5.9)

- `^` groq - update genai to 0.1.19 for Groq deepseek-r1-distill-llama-70b

## 2025-01-23 - [v0.5.8](https://github.com/jeremychone/rust-devai/compare/v0.5.7...v0.5.8)

- `^` genai - use genai v0.1.18 for local and remote deepseek support

## 2025-01-23 - [v0.5.7](https://github.com/jeremychone/rust-devai/compare/v0.5.6...v0.5.7)

- `-` (#24) fix - Compile - does not compile in non macos

## 2025-01-20 -  [v0.5.6](https://github.com/jeremychone/rust-devai/compare/v0.5.4...v0.5.6)

IMPORTANT: Can't compile on non-Mac. See v0.5.7 for fix. 

**v0.5.6**

- `-` init - fix issue when running without an devai (was hanging)

**v0.5.4**

- `+` NEW - agent - added the craft/[text,code] in default agents
- `+` NEW - agent module - added first support of `my_dir/my_agent.devai` support, `devai run my_dir/my_agent`
- `^` BIG - lua - big error reporting update (inline code line with issue)
- `-` FIX - init - fix to avoid recreating default .lua file on each init (when exists)
- `-` FIX - auth - made keyring only for mac (as it is supposed to be for now)
- `+` NEW - lua - add utils.text.split_first(content, sep)
- `-` lua - fix input not being 'nil' when it is not specified (now it is nil)
- `^` lua - functions optimization and fixes.
- `.` doc - fix doc/lua for CTX

## 2025-01-06 - `0.5.4`

- `+` NEW - ~/.devai-base/ - first pass (supports custom/agent and custom/lua)
- `+` NEW - lua - first pass at supporting 'require' with the '.devai/custom/lua' path added
- `!` CHANGE - remove devai new-solo
- `!` CHANGE - .devai/... name change, rename the  folders to  (for simplification)
    - e.g., Now `.devai/custom/agent` (before `.devai/custom/command-agent`)
- `.` init - do not create custom/new-template anymore
- `.` fix agent proof-comments
- `^` genai - updated to 0.1.17 with DeepSeek support
- `.` add in cargo.toml comment gemini-2.0-flash-exp
- `-` fix glob issue when relatively globs does not start with './'
- `.` update genai to 0.1.16
- `^` lua - override global lua print to print correctly
- `.` minor code refactor
- `.` lua_engine - minor refactor
- `.` clippy clean


## 2024-12-12 - `0.5.3`

Thanks to [Kees Jongenburger](https://github.com/keesj) for reporting 

- `-` Fix critical bug - [#23 cli issue - devai init fails when the .devai directory does not exits](https://github.com/jeremychone/rust-devai/issues/23)

## 2024-12-11 - `0.5.2`

> NOTE - This version introduced a critical bug (when .devai/ did not exist). 
         See [#23](https://github.com/jeremychone/rust-devai/issues/23)
         Use `0.5.3` and above

- `+` lua - add `utils.file.ensure_exists(path, optional_content)`
- `+` version - added `.devai/verion.txt` to force update doc on version change.
- `.` doc - remove ; in lua code
- `+` lua - add `utils.text.ensure(content, {prefix, suffix})`

## 2024-12-08 - `0.5.1`

- `+` Add xAI support (thanks to genai v0.1.15)
- `-` First fix for the keychain prompt
- `^` lua - load_md_sections now can take only the path (selecting all md sections)

## 2024-12-05 - `0.5.0`

- `*` BIG release with Lua and more. See [YouTube intro](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)
