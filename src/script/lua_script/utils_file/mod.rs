// region:    --- Modules

mod file_common;
mod file_md;

use crate::run::RuntimeContext;
use crate::script::lua_script::utils_file::file_common::{file_first, file_list, file_load, file_save};
use crate::script::lua_script::utils_file::file_md::{file_load_md_sections, file_load_md_split_first};
use crate::Result;
use mlua::{Lua, Table, Value};

// endregion: --- Modules

pub fn init_module(lua: &Lua, runtime_context: &RuntimeContext) -> Result<Table> {
	let table = lua.create_table()?;

	// -- load
	let ctx = runtime_context.clone();
	let file_load_fn = lua.create_function(move |lua, (path,): (String,)| file_load(lua, &ctx, path))?;

	// -- save
	let ctx = runtime_context.clone();
	let file_save_fn =
		lua.create_function(move |lua, (path, content): (String, String)| file_save(lua, &ctx, path, content))?;

	// -- list
	let ctx = runtime_context.clone();
	let file_list_fn = lua.create_function(move |lua, (glob,): (String,)| file_list(lua, &ctx, glob))?;

	// -- first
	let ctx = runtime_context.clone();
	let file_first_fn = lua.create_function(move |lua, (glob,): (String,)| file_first(lua, &ctx, glob))?;

	// -- load_md_sections
	let ctx = runtime_context.clone();
	let file_load_md_sections_fn = lua.create_function(move |lua, (path, headings): (String, Value)| {
		file_load_md_sections(lua, &ctx, path, headings)
	})?;

	// -- load_md_split_first
	let ctx = runtime_context.clone();
	let file_load_md_split_first_fn =
		lua.create_function(move |lua, (path,): (String,)| file_load_md_split_first(lua, &ctx, path))?;

	// -- All all function to the module
	table.set("load", file_load_fn)?;
	table.set("save", file_save_fn)?;
	table.set("list", file_list_fn)?;
	table.set("first", file_first_fn)?;
	table.set("load_md_sections", file_load_md_sections_fn)?;
	table.set("load_md_split_first", file_load_md_split_first_fn);

	Ok(table)
}
