use crate::Result;
use crate::agent::PartialAgentRef;
use crate::cli::ListArgs;
use crate::run::{DirContext, find_pack_dirs};
use crate::tui::print_pack_list;

pub async fn exec_list(dir_context: DirContext, list_args: ListArgs) -> Result<()> {
	// -- extract the optional namespace / pack_name from the args
	let (ns, pack_name) = if let Some(pack_ref) = list_args.pack_ref {
		// if no, @, then, assume itis the namespace
		// TODO: Handle the case where we have some special char in namespace
		if !pack_ref.contains('@') {
			(Some(pack_ref), None)
		} else if let PartialAgentRef::PackRef(pack_ref) = PartialAgentRef::new(&pack_ref) {
			let pack_name = pack_ref.name;
			let pack_name = if pack_name.trim().is_empty() {
				None
			} else {
				Some(pack_name)
			};
			(pack_ref.namespace, pack_name)
		} else {
			(None, None)
		}
	} else {
		(None, None)
	};

	let pack_dirs = find_pack_dirs(&dir_context, ns.as_deref(), pack_name.as_deref())?;

	print_pack_list(&pack_dirs);

	Ok(())
}
