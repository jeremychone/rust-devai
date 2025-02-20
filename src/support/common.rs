/// The type of "extrude" to be performed.
/// - `Content`   Concatenate all lines outside of marked blocks into one string.
/// - `Fragments` (NOT SUPPORTED YET): Have a vector of strings for Before, In Between, and After
#[derive(Debug, Clone, Copy)]
pub enum Extrude {
	Content,
}
