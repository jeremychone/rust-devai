`.` minor | `-` Fix | `+` Addition | `^` improvement | `!` Change | `*` Refactor

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
