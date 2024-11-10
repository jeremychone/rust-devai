use crate::{Error, Result};
use serde_json::Value;
use value_ext::JsonValueExt;

/// Custom data that can be returned by the Rhai script for special actions
#[derive(Debug, strum::AsRefStr)]
pub enum DevaiCustom {
	/// Will
	ActionSkip { reason: Option<String> },
	BeforeAllResponse {
		items: Option<Vec<Value>>,
		before_all: Option<Value>,
	},
}

/// Return of the `DevaiCustom::from_value` allowing to avoid cloning in case it's not a DevaiCustom.
#[derive(Debug)]
pub enum FromValue {
	DevaiCustom(DevaiCustom),
	OriginalValue(Value),
}

impl DevaiCustom {
	/// Check if the value is a `_devai_` Custom.
	///
	/// - if it is, it will parse and return the DevaiCusto
	/// - Otherwise, will return the original value
	/// - The formating of the `_devai_` action is as follow (example for skip action)
	///
	/// - The ActionSkip is as follow
	/// ```
	/// {
	///   _devai_: {
	///     kind: "ActionSkip", // or BeforeAllData
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
	///   _devai_: {
	///     kind: "BeforeAllResponse", // or BeforeAllData
	///     data: { // data is objectional, and each item is options.
	///       "items": ["item 1", "item 2", {some: "item 3"}],
	///       "before_all": {somee: "data, can be string, number or anything"}
	///     }
	///   }
	/// }
	/// ```
	///
	pub fn from_value(value: Value) -> Result<FromValue> {
		let Some(kind) = value.x_get::<String>("/_devai_/kind").ok() else {
			return Ok(FromValue::OriginalValue(value));
		};

		if kind == "ActionSkip" {
			let reason: Option<String> = value.x_get("/_devai_/data/reason").ok();
			Ok(FromValue::DevaiCustom(Self::ActionSkip { reason }))
		} else if kind == "BeforeAllResponse" {
			let custom_data: Option<Value> = value.x_get("/_devai_/data").ok();
			let (items, before_all) = extract_items_and_before_all(custom_data)?;
			Ok(FromValue::DevaiCustom(DevaiCustom::BeforeAllResponse {
				items,
				before_all,
			}))
		} else {
			Err(format!("_devai_ kind '{kind}' is not known.").into())
		}
	}
}

// region:    --- Support

fn extract_items_and_before_all(custom_data: Option<Value>) -> Result<(Option<Vec<Value>>, Option<Value>)> {
	let Some(custom_data) = custom_data else {
		return Ok((None, None));
	};

	const ERROR_CAUSE: &str = "devai::before_all_response(data), can only have `.item` and `.before_all`)";

	let (items, before_all) = match custom_data {
		Value::Object(mut obj) => {
			let before_all_data = obj.remove("before_all");
			let after_all_items = obj.remove("items");

			let items = match after_all_items {
				Some(Value::Array(new_items)) => Some(new_items),
				// if return items: Null, then will be None, which will have one item of Null below
				// > Note to cancel run, we will allow return {_devai_: {action: "skip"}} (not supported for now)
				Some(Value::Null) => None,
				Some(_) => {
					return Err(Error::BeforeAllFailWrongReturn {
						cause: "devai::before_all_response data .items must be an Null or an Array".to_string(),
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
			(items, before_all_data)
		}
		_ => (None, None),
	};

	Ok((items, before_all))
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
	fn test_devai_custom_before_all_items() -> Result<()> {
		// -- Setup & Fixtures
		let fx_custom = json!({
			"_devai_": {
				"kind": "BeforeAllResponse",
				"data": {
					"items": ["A", "B", 123],
					"before_all": "Some before all data"
				}
			}
		});

		// -- Exec
		let custom = DevaiCustom::from_value(fx_custom)?;

		// -- Check
		let FromValue::DevaiCustom(custom) = custom else {
			return Err("Should be a devai custom".into());
		};
		// lazy check
		let debug_string = format!("{:?}", custom);
		assert_contains(&debug_string, r#"items: Some([String("A"), String("B"), Number(123)]"#);
		assert_contains(&debug_string, r#"before_all: Some(String("Some before all data"))"#);

		Ok(())
	}
}

// endregion: --- Tests
