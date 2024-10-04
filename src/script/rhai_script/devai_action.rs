use serde_json::Value;
use value_ext::JsonValueExt;

pub enum DevaiAction {
	Skip { reason: Option<String> },
}

impl DevaiAction {
	/// Returns the eventual DevAction mathing this json structure
	///
	/// - The formating of a action is as follow (example for skip action)
	/// ```
	/// {
	///   _devai_: {
	///     action: "skip",
	///     data: { // optional
	///       "reason": "Some optional reaonse"
	///     }
	///   }
	/// }
	/// ```
	/// - For now, only support skip
	///
	/// TODO: Probably need to return `Result<Option<Self>>` to handle when the action is not valid
	pub fn from_value(value: &Value) -> Option<Self> {
		let action: String = value.x_get("/_devai_/action").ok()?;

		if action == "skip" {
			let reason: Option<String> = value.x_get("/_devai_/data/reason").ok();
			Some(Self::Skip { reason })
		} else {
			None
		}
	}
}
