use mlua::Value;

/// Convenient Lua Value extension
///
/// TODO: Will need to handle the case where the found value is not of correct type. Probably should return `Result<Option<>>`
#[allow(unused)]
pub trait LuaValueExt {
	fn x_get_string(&self, key: &str) -> Option<String>;
	fn x_get_bool(&self, key: &str) -> Option<bool>;
	fn x_get_i64(&self, key: &str) -> Option<i64>;
	fn x_get_f64(&self, key: &str) -> Option<f64>;
}

impl LuaValueExt for Value {
	fn x_get_string(&self, key: &str) -> Option<String> {
		let table = self.as_table()?;
		let val = table.get::<Value>(key).ok()?;
		let val = val.as_str()?;
		Some(val.to_string())
	}

	fn x_get_bool(&self, key: &str) -> Option<bool> {
		let table = self.as_table()?;
		let val = table.get::<Value>(key).ok()?;
		let val = val.as_boolean()?;
		Some(val)
	}

	fn x_get_i64(&self, key: &str) -> Option<i64> {
		let table = self.as_table()?;
		let val = table.get::<Value>(key).ok()?;
		let val = val.as_i64()?;
		Some(val)
	}

	fn x_get_f64(&self, key: &str) -> Option<f64> {
		let table = self.as_table()?;
		let val = table.get::<Value>(key).ok()?;
		let val = val.as_f64()?;
		Some(val)
	}
}

impl LuaValueExt for Option<Value> {
	fn x_get_string(&self, key: &str) -> Option<String> {
		self.as_ref()?.x_get_string(key)
	}

	fn x_get_bool(&self, key: &str) -> Option<bool> {
		self.as_ref()?.x_get_bool(key)
	}

	fn x_get_i64(&self, key: &str) -> Option<i64> {
		self.as_ref()?.x_get_i64(key)
	}

	fn x_get_f64(&self, key: &str) -> Option<f64> {
		self.as_ref()?.x_get_f64(key)
	}
}
