use derive_more::derive::From;
use rhai::Dynamic;
use std::any::Any;

pub struct DynamicMap(rhai::Map);

/// Constructors & Transformers
impl DynamicMap {
	pub fn new(dynamic: Dynamic) -> Result<DynamicMap, DynamicSupportError> {
		let map = dynamic.try_cast::<rhai::Map>().ok_or(DynamicSupportError::CastFailNotAMap)?;

		Ok(DynamicMap(map))
	}

	pub fn into_dynamic(self) -> Dynamic {
		Dynamic::from(self.0)
	}
}

impl DynamicMap {
	pub fn get<T: Any + Clone>(&self, name: &str) -> Result<T, DynamicSupportError> {
		let map = &self.0;

		let val = map.get(name).ok_or_else(|| DynamicSupportError::PropertyMissing {
			name: name.to_string(),
			cause: "missing".to_string(),
		})?;

		// map.get("name").ok_or("missing name property")?.clone().cast::<String>()
		let res = val.clone().cast::<T>();

		Ok(res)
	}
}

// region:    --- DynamicSupportExt

#[derive(Debug, From)]
pub enum DynamicSupportError {
	#[from]
	Custom(String),

	CastFailNotAMap,

	PropertyMissing {
		name: String,
		cause: String,
	}, // TBC
}

impl DynamicSupportError {
	pub fn custom(val: impl std::fmt::Display) -> Self {
		Self::Custom(val.to_string())
	}
}

impl From<&str> for DynamicSupportError {
	fn from(val: &str) -> Self {
		Self::Custom(val.to_string())
	}
}

impl core::fmt::Display for DynamicSupportError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for DynamicSupportError {}

// endregion: --- DynamicSupportExt
