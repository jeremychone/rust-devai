use crate::{Error, Result};
use serde_json::Value;

pub fn into_values<T: serde::Serialize>(vals: Vec<T>) -> Result<Vec<Value>> {
	let inputs: Vec<Value> = vals
		.into_iter()
		.map(|v| serde_json::to_value(v).map_err(Error::custom))
		.collect::<Result<Vec<_>>>()?;

	Ok(inputs)
}
