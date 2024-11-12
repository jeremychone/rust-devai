use rhai::Dynamic;

pub trait IntoDynamic {
	fn into_dynamic(self) -> Dynamic;
}

// region:    --- Blanket Implementations

impl<T: IntoDynamic> IntoDynamic for Vec<T> {
	fn into_dynamic(self) -> Dynamic {
		let mut array = rhai::Array::new();
		for item in self {
			array.push(item.into_dynamic());
		}
		Dynamic::from(array)
	}
}

// impl for Option<T>
impl<T: IntoDynamic> IntoDynamic for Option<T> {
	fn into_dynamic(self) -> Dynamic {
		match self {
			Some(value) => value.into_dynamic(),
			None => Dynamic::UNIT,
		}
	}
}

// endregion: --- Blanket Implementations

// region:    --- String Impls

impl IntoDynamic for &str {
	fn into_dynamic(self) -> Dynamic {
		Dynamic::from(self.to_string())
	}
}

impl IntoDynamic for &String {
	fn into_dynamic(self) -> Dynamic {
		Dynamic::from(self.to_string())
	}
}

impl IntoDynamic for String {
	fn into_dynamic(self) -> Dynamic {
		Dynamic::from(self)
	}
}

// endregion: --- String Impls

// region:    --- Common Implementations

macro_rules! int_impl_into_dynamic {
    ($($t:ty),*) => {
        $(
            impl IntoDynamic for $t {
                fn into_dynamic(self) -> Dynamic {
                    Dynamic::from(self as i64)
                }
            }
        )*
    }
}

macro_rules! float_impl_into_dynamic {
    ($($t:ty),*) => {
        $(
            impl IntoDynamic for $t {
                fn into_dynamic(self) -> Dynamic {
                    Dynamic::from(self as f64)
                }
            }
        )*
    }
}

int_impl_into_dynamic!(u16, u32, i32, i64, usize);
float_impl_into_dynamic!(f32, f64);

impl IntoDynamic for bool {
	fn into_dynamic(self) -> Dynamic {
		Dynamic::from(self)
	}
}

// endregion: --- Common Implementations

// region:    --- Other Implementations

/// Comment for now, to spot uncessary into_dynamic
// impl IntoDynamic for Dynamic {
// 	fn into_dynamic(self) -> Dynamic {
// 		self
// 	}
// }

impl IntoDynamic for () {
	fn into_dynamic(self) -> Dynamic {
		Dynamic::UNIT
	}
}

// endregion: --- Other Implementations
