use crate::{Error, Result};
use serde_json::Value;

/// Utility extension trait for anything that contains a list of vaalues
pub trait ValuesExt: Sized {
	fn x_into_values(self) -> Result<Vec<Value>>;
}

impl<T> ValuesExt for Vec<T>
where
	T: serde::Serialize,
{
	fn x_into_values(self) -> Result<Vec<Value>> {
		let items: Vec<Value> = self
			.into_iter()
			.map(|v| serde_json::to_value(v).map_err(Error::custom))
			.collect::<Result<Vec<_>>>()?;

		Ok(items)
	}
}

impl ValuesExt for Value {
	fn x_into_values(self) -> Result<Vec<Value>> {
		match self {
			Value::Array(arr) => Ok(arr),
			_ => Err(Error::custom("Expected a JSON array")),
		}
	}
}
