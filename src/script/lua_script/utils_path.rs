//! Defines the `path` module, used in the lua engine.
//!
//! ---
//!
//! ## LUA documentation
//! The `path` module exposes functions used to interact with file paths.
//!
//! ### Functions
//! * `path::exists(path: string) -> bool`
//! * `path::is_file(path: string) -> bool`
//! * `path::is_dir(path: string) -> bool`
//! * `path::parent(path: string) -> string | nil`

use crate::run::{PathResolver, RuntimeContext};
use mlua::{Lua, Table};
use std::path::Path;
use crate::Result;

pub fn init_module(lua: &Lua, runtime_context: &RuntimeContext) -> Result<Table> {
    let table = lua.create_table()?;

    // -- exists
    let ctx = runtime_context.clone();
    let path_exists_fn = lua.create_function(move |_lua, path: String| path_exists(&ctx, path))?;

    // -- is_file  
    let ctx = runtime_context.clone();
    let path_is_file_fn = lua.create_function(move |_lua, path: String| path_is_file(&ctx, path))?;

    // -- is_dir
    let ctx = runtime_context.clone();
    let path_is_dir_fn = lua.create_function(move |_lua, path: String| path_is_dir(&ctx, path))?;

    // -- parent
    let path_parent_fn = lua.create_function(move |_lua, path: String| path_parent(path))?;

    // -- Add all functions to the module
    table.set("exists", path_exists_fn)?;
    table.set("is_file", path_is_file_fn)?;
    table.set("is_dir", path_is_dir_fn)?;
    table.set("parent", path_parent_fn)?;

    Ok(table)
}

// region:    --- Lua Functions

/// ## LUA Documentation
/// ```lua
/// path.exists(path: string) -> bool
/// ```
///
/// Checks if the specified path exists.
fn path_exists(ctx: &RuntimeContext, path: String) -> mlua::Result<bool> {
    let path = ctx.dir_context().resolve_path(&path, PathResolver::DevaiParentDir)?;
    Ok(path.exists())
}

/// ## LUA Documentation
/// ```lua
/// path.is_file(path: string) -> bool
/// ```
///
/// Checks if the specified path is a file.
fn path_is_file(ctx: &RuntimeContext, path: String) -> mlua::Result<bool> {
    let path = ctx.dir_context().resolve_path(&path, PathResolver::DevaiParentDir)?;
    Ok(path.is_file())
}

/// ## LUA Documentation
/// ```lua
/// path.is_dir(path: string) -> bool
/// ```
///
/// Checks if the specified path is a directory.
fn path_is_dir(ctx: &RuntimeContext, path: String) -> mlua::Result<bool> {
    let path = ctx.dir_context().resolve_path(&path, PathResolver::DevaiParentDir)?;
    Ok(path.is_dir())
}

/// ## LUA Documentation
/// ```lua
/// path.parent(path: string) -> string | nil
/// ```
///
/// Returns the parent directory of the specified path, or nil if there is no parent.
/// (follows the Rust Path::parent(&self) logic)
fn path_parent(path: String) -> mlua::Result<Option<String>> {
    match Path::new(&path).parent() {
        Some(parent) => match parent.to_str() {
            Some(parent_str) => Ok(Some(parent_str.to_string())),
            None => Ok(None),
        },
        None => Ok(None),
    }
}

// endregion: --- Lua Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
    //! NOTE 1: Here we are testing these functions in the context of an agent to ensure they work in that context.
    //!         A more purist approach would be to test the Lua functions in a blank Lua engine, but the net value of testing
    //!         them in the context where they will run is higher. Height wins.
    //!
    //! NOTE 2: All the tests here are with run_reflective_agent that have the tests-data/sandbox-01 as current dir.
    type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

    use crate::_test_support::run_reflective_agent;

    #[tokio::test]
    async fn test_lua_path_exists_true() -> Result<()> {
        // -- Fixtures
        let paths = &[
            //
            "./agent-script/agent-hello.devai",
            "agent-script/agent-hello.devai", 
            "./sub-dir-a/agent-hello-2.devai",
            "sub-dir-a/agent-hello-2.devai",
            "sub-dir-a/",
            "sub-dir-a",
            "./sub-dir-a/",
            "./sub-dir-a/../",
            "./sub-dir-a/..",
        ];

        // -- Exec & Check
        for path in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.exists("{path}")"#), None).await?;
            assert!(
                res.as_bool().ok_or("Result should be a bool")?,
                "'{path}' should exists"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lua_path_exists_false() -> Result<()> {
        // -- Fixtures
        let paths = &[
            //
            "./no file .rs",
            "some/no-file.md",
            "./s do/",
            "no-dir/at/all",
        ];

        // -- Exec & Check
        for path in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.exists("{path}")"#), None).await?;
            assert!(
                !res.as_bool().ok_or("Result should be a bool")?,
                "'{path}' should NOT exists"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lua_path_is_file_true() -> Result<()> {
        // -- Fixtures
        let paths = &[
            //
            "./agent-script/agent-hello.devai",
            "agent-script/agent-hello.devai",
            "./sub-dir-a/agent-hello-2.devai", 
            "sub-dir-a/agent-hello-2.devai",
            "sub-dir-a/../agent-script/agent-hello.devai",
        ];

        // -- Exec & Check
        for path in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.is_file("{path}")"#), None).await?;
            assert!(
                res.as_bool().ok_or("Result should be a bool")?,
                "'{path}' should be is_file"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lua_path_is_file_false() -> Result<()> {
        // -- Fixtures
        let paths = &[
            //
            "./no-file",
            "no-file.txt",
            "sub-dir-a/",
        ];

        // -- Exec & Check
        for path in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.is_file("{path}")"#), None).await?;
            assert!(
                !res.as_bool().ok_or("Result should be a bool")?,
                "'{path}' should NOT be is_file"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lua_path_is_dir_true() -> Result<()> {
        // -- Fixtures
        let paths = &[
            //
            "./sub-dir-a",
            "sub-dir-a",
            "./sub-dir-a/..",
            // Note: below does not work for now becsuse some-other-dir does not exists. Might want to use clean.
            // "./sub-dir-a/some-other-dir/..",
        ];

        // -- Exec & Check
        for path in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.is_dir("{path}")"#), None).await?;
            assert!(
                res.as_bool().ok_or("Result should be a bool")?,
                "'{path}' should be is_dir"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lua_path_is_dir_false() -> Result<()> {
        // -- Fixtures
        let paths = &[
            //
            "./agent-hello.devai",
            "agent-hello.devai",
            "./sub-dir-a/agent-hello-2.devai",
            "./sub-dir-a/other-path",
            "nofile.txt",
            "./s rc/",
        ];

        // -- Exec & Check
        for path in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.is_dir("{path}")"#), None).await?;
            assert!(
                !res.as_bool().ok_or("Result should be a bool")?,
                "'{path}' should NOT be is_dir"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_lua_path_parent() -> Result<()> {
        // -- Fixtures
        // This is the rust Path logic
        let paths = &[
            //
            ("./agent-hello.devai", "."),
            ("./", ""),
            (".", ""),
            ("./sub-dir/file.txt", "./sub-dir"),
            ("./sub-dir/file", "./sub-dir"),
            ("./sub-dir/", "."),
            ("./sub-dir", "."),
        ];

        // -- Exec & Check
        for (path, expected) in paths {
            let res = run_reflective_agent(&format!(r#"return utils.path.parent("{path}")"#), None).await?;
            let res = res.as_str().ok_or("Should be a string")?;
            assert_eq!(res, *expected);
        }

        Ok(())
    }
}

// endregion: --- Tests
