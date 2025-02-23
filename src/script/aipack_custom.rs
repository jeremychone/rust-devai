use crate::{Error, Result};
use serde_json::Value;
use value_ext::JsonValueExt;

/// Custom data that can be returned by the lua script for special actions
#[derive(Debug, strum::AsRefStr)]
pub enum AipackCustom {
	/// Will
	Skip {
		reason: Option<String>,
	},
	BeforeAllResponse(BeforeAllResponse),
}

#[derive(Debug, Default)]
pub struct BeforeAllResponse {
	pub inputs: Option<Vec<Value>>,
	pub before_all: Option<Value>,
	pub options: Option<Value>,
}

/// Return of the `AipackCustom::from_value` allowing to avoid cloning in case it's not a AipackCustom.
#[derive(Debug)]
pub enum FromValue {
	AipackCustom(AipackCustom),
	OriginalValue(Value),
}

impl AipackCustom {
	/// Check if the value is a `_aipack_` Custom.
	///
	/// - if it is, it will parse and return the AipackCustom
	/// - Otherwise, will return the original value
	/// - The formating of the `_aipack_` action is as follow (example for skip action)
	///
	/// - The Skip is as follow
	/// ```
	/// {
	///   _aipack_: {
	///     kind: "Skip", // or BeforeAllData
	///     data: { // optional
	///       "reason": "Some optional reason"
	///     }
	///   }
	/// }
	/// ```
	///
	/// - The BeforeAllResponse
	/// ```
	/// {
	///   _aipack_: {
	///     kind: "BeforeAllResponse", // or BeforeAllData
	///     data: { // data is objectional, and each input is options.
	///       "inputs": ["input 1", "input 2", {some: "input 3"}],
	///       "before_all": {somee: "data, can be string, number or anything"}
	///     }
	///   }
	/// }
	/// ```
	///
	pub fn from_value(value: Value) -> Result<FromValue> {
		let Some(kind) = value.x_get::<String>("/_aipack_/kind").ok() else {
			return Ok(FromValue::OriginalValue(value));
		};

		if kind == "Skip" {
			let reason: Option<String> = value.x_get("/_aipack_/data/reason").ok();
			Ok(FromValue::AipackCustom(Self::Skip { reason }))
		} else if kind == "BeforeAllResponse" {
			let custom_data: Option<Value> = value.x_get("/_aipack_/data").ok();
			let before_all_response = extract_inputs_and_before_all(custom_data)?;
			Ok(FromValue::AipackCustom(AipackCustom::BeforeAllResponse(
				before_all_response,
			)))
		} else {
			Err(format!("_aipack_ kind '{kind}' is not known.").into())
		}
	}
}

// region:    --- Support

/// extract, (inputs, before_all_data, options)
fn extract_inputs_and_before_all(custom_data: Option<Value>) -> Result<BeforeAllResponse> {
	let Some(custom_data) = custom_data else {
		return Ok(BeforeAllResponse::default());
	};

	const ERROR_CAUSE: &str = "aipack::before_all_response(data), can only have `.input` and `.before_all`)";

	let before_all_response = match custom_data {
		Value::Object(mut obj) => {
			let all_inputs = obj.remove("inputs");
			let before_all = obj.remove("before_all");
			let options = obj.remove("options");

			let inputs = match all_inputs {
				Some(Value::Array(new_inputs)) => Some(new_inputs),
				// if return inputs: Null, then will be None, which will have one input of Null below
				// > Note to cancel run, we will allow return {_aipack_: {action: "skip"}} (not supported for now)
				Some(Value::Null) => None,
				Some(_) => {
					return Err(Error::BeforeAllFailWrongReturn {
						cause: "aipack::before_all_response data .inputs must be an Null or an Array".to_string(),
					});
				}
				None => None,
			};

			let keys: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
			if !keys.is_empty() {
				return Err(Error::BeforeAllFailWrongReturn {
					cause: format!("{ERROR_CAUSE}. But also contained: {}", keys.join(", ")),
				});
			}
			BeforeAllResponse {
				inputs,
				before_all,
				options,
			}
		}
		_ => BeforeAllResponse::default(),
	};

	Ok(before_all_response)
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::assert_contains;
	use serde_json::json;

	#[test]
	fn test_aipack_custom_before_all_inputs() -> Result<()> {
		// -- Setup & Fixtures
		let fx_custom = json!({
			"_aipack_": {
				"kind": "BeforeAllResponse",
				"data": {
					"inputs": ["A", "B", 123],
					"before_all": "Some before all data"
				}
			}
		});

		// -- Exec
		let custom = AipackCustom::from_value(fx_custom)?;

		// -- Check
		let FromValue::AipackCustom(custom) = custom else {
			return Err("Should be a aipack custom".into());
		};
		// lazy check
		let debug_string = format!("{:?}", custom);
		assert_contains(&debug_string, r#"inputs: Some([String("A"), String("B"), Number(123)]"#);
		assert_contains(&debug_string, r#"before_all: Some(String("Some before all data"))"#);

		Ok(())
	}
}

// endregion: --- Tests
