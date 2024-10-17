use crate::{Error, Result};
use serde_json::Value;
use value_ext::JsonValueExt;

pub enum DevaiCustom {
	ActionSkip {
		reason: Option<String>,
	},
	BeforeAll {
		items: Option<Vec<Value>>,
		before_all: Option<Value>,
	},
}

impl DevaiCustom {
	/// A data structure returned by the Rhai script to perform some custom action or behavior.
	///
	/// - The formating of a action is as follow (example for skip action)
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
	/// - For now, only support skip
	///
	/// TODO: Probably need to return `Result<Option<Self>>` to handle when the action is not valid
	pub fn from_value(value: &Value) -> Result<Option<Self>> {
		let Some(kind) = value.x_get::<String>("/_devai_/kind").ok() else {
			return Ok(None);
		};

		if kind == "ActionSkip" {
			let reason: Option<String> = value.x_get("/_devai_/data/reason").ok();
			Ok(Some(Self::ActionSkip { reason }))
		} else if kind == "BeforeAll" {
			let custom_data: Option<Value> = value.x_get("/_devai_/data").ok();
			let (items, before_all) = extract_items_and_before_all(custom_data)?;
			Ok(Some(DevaiCustom::BeforeAll { items, before_all }))
		} else {
			Ok(None)
		}
	}
}

// region:    --- Support

fn extract_items_and_before_all(custom_data: Option<Value>) -> Result<(Option<Vec<Value>>, Option<Value>)> {
	let Some(custom_data) = custom_data else {
		return Ok((None, None));
	};

	let (items, before_all) = match custom_data {
		Value::Object(mut obj) => {
			let after_all_items = obj.remove("items");

			let items = match after_all_items {
				Some(Value::Array(new_items)) => Some(new_items),
				// if return items: Null, then will be None, which will have one item of Null below
				// > Note to cancel run, we will allow return {_devai_: {action: "skip"}} (not supported for now)
				Some(Value::Null) => None,
				Some(_) => {
					return Err(Error::BeforeAllFailWrongReturn {
                        cause: "Before All script block, return `.items` is not type Array, must be an array (even Array of one if one item)".to_string()
                    });
				}
				None => None,
			};

			let before_all_data = obj.remove("before_all_data");
			let keys: Vec<String> = obj.keys().map(|k| k.to_string()).collect();
			if !keys.is_empty() {
				return Err(Error::BeforeAllFailWrongReturn {
                        cause: format!("Before All script block, can only return '.items' and/or '.before_all_data' but also returned {}", keys.join(", "))
                    });
			}
			(items, before_all_data)
		}
		_ => (None, None),
	};

	Ok((items, before_all))
}

// endregion: --- Support
