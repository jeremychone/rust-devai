`.` minor | `-` Fix | `+` Addition | `^` improvement | `!` Change | `*` Refactor

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
