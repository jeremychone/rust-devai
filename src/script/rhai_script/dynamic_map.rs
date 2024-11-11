use derive_more::derive::From;
use rhai::Dynamic;
use std::any::Any;

#[derive(Default)]
pub struct DynamicMap(rhai::Map);

/// Constructors & Transformers
impl DynamicMap {
	pub fn from_dynamic(dynamic: Dynamic) -> Result<DynamicMap, DynamicSupportError> {
		let map = dynamic.try_cast::<rhai::Map>().ok_or(DynamicSupportError::CastFailNotAMap)?;

		Ok(DynamicMap(map))
	}

	#[allow(unused)]
	pub fn into_dynamic(self) -> Dynamic {
		Dynamic::from(self.0)
	}
}

impl DynamicMap {
	pub fn insert(mut self, name: &'static str, value: impl Into<Dynamic>) -> Self {
		self.0.insert(name.into(), value.into());
		self
	}

	/// Convenient function to get a property as a type
	/// NOTE: Today, will clone the value
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

	#[allow(unused)]
	pub fn remove_to_dynamic(&mut self, name: &str) -> Result<Option<Dynamic>, DynamicSupportError> {
		let map = &mut self.0;

		let val = map.remove(name);

		Ok(val)
	}
}

impl From<DynamicMap> for Dynamic {
	fn from(val: DynamicMap) -> Self {
		val.into_dynamic()
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
	}, // To be confirmed
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
