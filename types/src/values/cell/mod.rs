mod path;

use std::{collections::{HashMap, HashSet}, fmt, hash::{Hash, Hasher}, ptr};
use crate::{EnumValueType, StructType, Type, U256};
use super::{Value, ValueError, SubValue, ValueType};

pub use path::*;

// Give inner mutability for values with inner types.
// This is NOT thread-safe due to the RefCell usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueCell {
    Default(Value),
    Struct(Vec<SubValue>, StructType),
    Array(Vec<SubValue>),
    Optional(Option<SubValue>),

    // Map cannot be used as a key in another map
    Map(HashMap<ValueCell, SubValue>),
    Enum(Vec<SubValue>, EnumValueType),
}

impl Hash for ValueCell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash_with_pointers(state, &mut HashSet::new());
    }
}

impl Default for ValueCell {
    fn default() -> Self {
        Self::Default(Default::default())
    }
}

impl From<Value> for ValueCell {
    fn from(value: Value) -> Self {
        Self::Default(value)
    }
}

impl From<ValueType> for ValueCell {
    fn from(value: ValueType) -> Self {
        match value {
            ValueType::Default(v) => Self::Default(v),
            ValueType::Struct(fields, _type) => Self::Struct(fields.into_iter().map(|v| v.into()).collect(), _type),
            ValueType::Array(values) => Self::Array(values.into_iter().map(|v| v.into()).collect()),
            ValueType::Optional(value) => Self::Optional(value.map(|v| (*v).into())),
            ValueType::Map(map) => Self::Map(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect()),
            ValueType::Enum(fields, _type) => Self::Enum(fields.into_iter().map(|v| v.into()).collect(), _type)
        }
    }
}

impl ValueCell {
    pub(crate) fn hash_with_pointers<H: Hasher>(&self, state: &mut H, tracked_pointers: &mut HashSet<*const Self>) {
        if !tracked_pointers.insert(ptr::from_ref(self)) {
            // Cyclic reference detected
            return;
        }

        match self {
            ValueCell::Default(v) => {
                v.hash(state);
            },
            ValueCell::Struct(fields, _) => {
                fields.iter()
                    .for_each(|field| field.borrow()
                        .hash_with_pointers(state, tracked_pointers)
                    );
            },
            ValueCell::Array(array) => {
                array.iter()
                .for_each(|field| field.borrow()
                    .hash_with_pointers(state, tracked_pointers)
                );
            },
            ValueCell::Optional(v) => {
                if let Some(v) = v {
                    v.borrow()
                        .hash_with_pointers(state, tracked_pointers);
                } else {
                    Self::Default(Value::Null).hash(state);
                }
            },
            ValueCell::Map(map) => {
                map.iter()
                    .for_each(|(k, v)| {
                        k.hash(state);
                        v.borrow()
                            .hash_with_pointers(state, tracked_pointers);
                    });
            },
            ValueCell::Enum(fields, _) => {
                fields.iter()
                    .for_each(|field| field.borrow()
                        .hash_with_pointers(state, tracked_pointers)
                    );
            }
        }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        match &self {
            Self::Default(Value::Null) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_string(&self) -> bool {
        match &self {
            Self::Default(Value::String(_)) => true,
            _ => false
        }
    }

    #[inline]
    pub fn is_map(&self) -> bool {
        match &self {
            Self::Map(_) => true,
            _ => false
        }
    }

    #[inline]
    pub fn as_u8(&self) -> Result<u8, ValueError> {
        match self {
            Self::Default(Value::U8(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U8))
        }
    }

    #[inline]
    pub fn as_u16(&self) -> Result<u16, ValueError> {
        match self {
            Self::Default(Value::U16(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U16))
        }
    }

    #[inline]
    pub fn as_u32(&self) -> Result<u32, ValueError> {
        match self {
            Self::Default(Value::U32(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U32))
        }
    }

    #[inline]
    pub fn as_u64(&self) -> Result<u64, ValueError> {
        match self {
            Self::Default(Value::U64(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U64))
        }
    }

    #[inline]
    pub fn as_u128(&self) -> Result<u128, ValueError> {
        match self {
            Self::Default(Value::U128(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U128))
        }
    }

    #[inline]
    pub fn as_u256(&self) -> Result<U256, ValueError> {
        match self {
            Self::Default(Value::U256(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U256))
        }
    }

    #[inline]
    pub fn as_string(&self) -> Result<&String, ValueError> {
        match self {
            Self::Default(Value::String(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::String))
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Result<bool, ValueError> {
        match self {
            Self::Default(Value::Boolean(n)) => Ok(*n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Bool))
        }
    }

    #[inline]
    pub fn as_map(&self) -> Result<&HashMap<Self, SubValue>, ValueError> {
        match self {
            Self::Map(map) => Ok(map),
            _ => Err(ValueError::ExpectedStruct)
        }
    }

    #[inline]
    pub fn as_mut_map(&mut self) -> Result<&mut HashMap<Self, SubValue>, ValueError> {
        match self {
            Self::Map(map) => Ok(map),
            _ => Err(ValueError::ExpectedStruct),
        }
    }

    #[inline]
    pub fn as_vec<'a>(&'a self) -> Result<&'a Vec<SubValue>, ValueError> {
        match self {
            Self::Array(n) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Array(Box::new(Type::Any))))
        }
    }

    #[inline]
    pub fn as_mut_vec<'a>(&'a mut self) -> Result<&'a mut Vec<SubValue>, ValueError> {
        match self {
            Self::Array(n) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Array(Box::new(Type::Any))))
        }
    }

    #[inline]
    pub fn as_optional(&self, expected: &Type) -> Result<Option<&SubValue>, ValueError> {
        match self {
            Self::Default(Value::Null) => Ok(None),
            Self::Optional(n) => Ok(n.as_ref()),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Optional(Box::new(expected.clone()))))
        }
    }

    #[inline]
    pub fn take_from_optional(&mut self, expected: &Type) -> Result<SubValue, ValueError> {
        match self {
            Self::Optional(opt) => opt.take().ok_or(ValueError::OptionalIsNull),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Optional(Box::new(expected.clone()))))
        }
    }

    #[inline]
    pub fn take_optional(&mut self) -> Result<Option<SubValue>, ValueError> {
        match self {
            Self::Optional(opt) => Ok(opt.take()),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Optional(Box::new(Type::Any))))
        }
    }

    #[inline]
    pub fn to_u8(self) -> Result<u8, ValueError> {
        match self {
            Self::Default(Value::U8(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U8))
        }
    }

    #[inline]
    pub fn to_u16(self) -> Result<u16, ValueError> {
        match self {
            Self::Default(Value::U16(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U16))
        }
    }

    #[inline]
    pub fn to_u32(self) -> Result<u32, ValueError> {
        match self {
            Self::Default(Value::U32(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U32))
        }
    }

    #[inline]
    pub fn to_u64(self) -> Result<u64, ValueError> {
        match self {
            Self::Default(Value::U64(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U64))
        }
    }

    #[inline]
    pub fn to_u128(self) -> Result<u128, ValueError> {
        match self {
            Self::Default(Value::U128(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U128))
        }
    }

    #[inline]
    pub fn to_u256(self) -> Result<U256, ValueError> {
        match self {
            Self::Default(Value::U256(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::U256))
        }
    }

    #[inline]
    pub fn to_string(self) -> Result<String, ValueError> {
        match self {
            Self::Default(Value::String(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::String))
        }
    }

    #[inline]
    pub fn to_bool(self) -> Result<bool, ValueError> {
        match self {
            Self::Default(Value::Boolean(n)) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Bool))
        }
    }

    #[inline]
    pub fn to_map(self) -> Result<Vec<SubValue>, ValueError> {
        match self {
            Self::Struct(fields, _) => Ok(fields),
            _ => Err(ValueError::ExpectedStruct)
        }
    }

    #[inline]
    pub fn to_vec(self) -> Result<Vec<SubValue>, ValueError> {
        match self {
            Self::Array(n) => Ok(n),
            v => Err(ValueError::InvalidValueCell(v.clone(), Type::Array(Box::new(Type::Any))))
        }
    }

    #[inline]
    pub fn to_sub_vec(self) -> Result<Vec<SubValue>, ValueError> {
        match self {
            Self::Array(values) => Ok(values),
            Self::Struct(fields, _) => Ok(fields),
            _ => Err(ValueError::SubValue)
        }
    }

    #[inline]
    pub fn as_sub_vec(&self) -> Result<&Vec<SubValue>, ValueError> {
        match self {
            Self::Array(values) => Ok(values),
            Self::Struct(fields, _) => Ok(fields),
            _ => Err(ValueError::SubValue)
        }
    }

    #[inline]
    pub fn as_mut_sub_vec(&mut self) -> Result<&mut Vec<SubValue>, ValueError> {
        match self {
            Self::Array(values) => Ok(values),
            Self::Struct(fields, _) => Ok(fields),
            _ => Err(ValueError::SubValue)
        }
    }

    #[inline]
    pub fn as_range(&self) -> Result<(&Value, &Value, &Type), ValueError> {
        self.as_value().and_then(Value::as_range)
    }

    #[inline]
    pub fn to_range(self) -> Result<(Value, Value, Type), ValueError> {
        self.into_value().and_then(Value::to_range)
    }

    // Check if the value is a number
    #[inline]
    pub fn is_number(&self) -> bool {
        match self {
            Self::Default(v) => v.is_number(),
            _ => false
        }
    }

    // Increment the value
    pub fn increment(&mut self) -> Result<(), ValueError> {
        Ok(match self {
            Self::Default(v) => v.increment()?,
            _ => return Err(ValueError::OperationNotNumberType)
        })
    }

    // Decrement the value
    pub fn decrement(&mut self) -> Result<(), ValueError> {
        Ok(match self {
            Self::Default(v) => v.decrement()?,
            _ => return Err(ValueError::OperationNotNumberType)
        })
    }

    // Cast value to string
    #[inline]
    pub fn cast_to_string(self) -> Result<String, ValueError> {
        match self {
            Self::Default(v) => v.cast_to_string(),
            _ => Err(ValueError::InvalidCastType(Type::String))
        }
    }

    // Cast the value to the expected type
    pub fn mut_checked_cast_to_primitive_type(&mut self, expected: &Type) -> Result<(), ValueError> {
        let take = std::mem::take(self);
        let value = take.checked_cast_to_primitive_type(expected)?;
        *self = value;
        Ok(())
    }

    // Cast without loss in the expected type
    #[inline]
    pub fn checked_cast_to_primitive_type(self, expected: &Type) -> Result<Self, ValueError> {
        match expected {
            Type::U8 => self.checked_cast_to_u8().map(Value::U8),
            Type::U16 => self.checked_cast_to_u16().map(Value::U16),
            Type::U32 => self.checked_cast_to_u32().map(Value::U32),
            Type::U64 => self.checked_cast_to_u64().map(Value::U64),
            Type::U128 => self.checked_cast_to_u128().map(Value::U128),
            Type::U256 => self.checked_cast_to_u256().map(Value::U256),
            Type::String => self.cast_to_string().map(Value::String),
            Type::Bool => self.cast_to_bool().map(Value::Boolean),
            Type::Optional(inner) => {
                if self.is_null() {
                    return Ok(Self::Optional(None))
                } else {
                    return self.checked_cast_to_primitive_type(inner)
                }
            },
            Type::Range(inner) => {
                let (start, end, _) = self.to_range()?;
                let start = start.checked_cast_to_primitive_type(inner)?;
                let end = end.checked_cast_to_primitive_type(inner)?;
                Ok(Value::Range(Box::new(start), Box::new(end), *inner.clone()))
            },
            _ => Err(ValueError::InvalidCastType(expected.clone()))
        }.map(Self::Default)
    }

    // Cast to u8, return an error if value is too big
    #[inline]
    pub fn checked_cast_to_u8(self) -> Result<u8, ValueError> {
        self.into_value().and_then(Value::checked_cast_to_u8)
    }

    // Cast to u16, return an error if value is too big
    #[inline]
    pub fn checked_cast_to_u16(self) -> Result<u16, ValueError> {
        self.into_value().and_then(Value::checked_cast_to_u16)
    }

    // Cast to u32, return an error if value is too big
    #[inline]
    pub fn checked_cast_to_u32(self) -> Result<u32, ValueError> {
        self.into_value().and_then(Value::checked_cast_to_u32)
    }

    // Cast to u64, return an error if value is too big
    #[inline]
    pub fn checked_cast_to_u64(self) -> Result<u64, ValueError> {
        self.into_value().and_then(Value::checked_cast_to_u64)
    }

    // Cast to u128, return an error if value is too big
    #[inline]
    pub fn checked_cast_to_u128(self) -> Result<u128, ValueError> {
        self.into_value().and_then(Value::checked_cast_to_u128)
    }

    // Cast to u256, return an error if value is too big
    #[inline]
    pub fn checked_cast_to_u256(self) -> Result<U256, ValueError> {
        self.into_value().and_then(Value::checked_cast_to_u256)
    }

    // Cast value to bool
    #[inline]
    pub fn cast_to_bool(self) -> Result<bool, ValueError> {
        self.into_value().and_then(Value::cast_to_bool)
    }

    // Cast value to u8
    #[inline]
    pub fn cast_to_u8(self) -> Result<u8, ValueError> {
        self.into_value().and_then(Value::cast_to_u8)
    }

    // Cast value to u16
    #[inline]
    pub fn cast_to_u16(self) -> Result<u16, ValueError> {
        self.into_value().and_then(Value::cast_to_u16)
    }

    // Cast value to u32
    #[inline]
    pub fn cast_to_u32(self) -> Result<u32, ValueError> {
        self.into_value().and_then(Value::cast_to_u32)
    }

    // Cast value to u64
    #[inline]
    pub fn cast_to_u64(self) -> Result<u64, ValueError> {
        self.into_value().and_then(Value::cast_to_u64)
    }

    // Cast value to u128
    #[inline]
    pub fn cast_to_u128(self) -> Result<u128, ValueError> {
        self.into_value().and_then(Value::cast_to_u128)
    }

    // Cast value to u256
    #[inline]
    pub fn cast_to_u256(self) -> Result<U256, ValueError> {
        self.into_value().and_then(Value::cast_to_u256)
    }

    #[inline(always)]
    pub fn as_value(&self) -> Result<&Value, ValueError> {
        match self {
            Self::Default(v) => Ok(v),
            _ => Err(ValueError::InvalidValueCell(self.clone(), Type::Any))
        }
    }

    #[inline(always)]
    pub fn into_value(self) -> Result<Value, ValueError> {
        match self {
            Self::Default(v) => Ok(v),
            _ => Err(ValueError::InvalidValueCell(self, Type::Any))
        }
    }
}

impl fmt::Display for ValueCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default(v) => write!(f, "{}", v),
            Self::Struct(fields, _type) => {
                let s: Vec<String> = fields.iter().enumerate().map(|(k, v)| format!("{}: {}", k, v.borrow())).collect();
                write!(f, "{:?} {} {} {}", _type, "{", s.join(", "), "}")
            },
            Self::Array(values) => {
                let s: Vec<String> = values.iter().map(|v| format!("{}", v.borrow())).collect();
                write!(f, "[{}]", s.join(", "))
            },
            Self::Optional(value) => match value.as_ref() {
                Some(value) => write!(f, "optional<{}>", value.borrow().to_string()),
                None => write!(f, "optional<null>")
            },
            Self::Map(map) => {
                let s: Vec<String> = map.iter().map(|(k, v)| format!("{}: {}", k, v.borrow())).collect();
                write!(f, "map{}{}{}", "{", s.join(", "), "}")
            },
            Self::Enum(fields, enum_type) => {
                let s: Vec<String> = fields.iter().enumerate().map(|(k, v)| format!("{}: {}", k, v.borrow())).collect();
                write!(f, "enum{:?} {} {} {}", enum_type, "{", s.join(", "), "}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SubValue;

    use super::*;

    #[test]
    fn test_stackoverflow_std_hash() {
        // Create a map that contains itself
        let map = SubValue::new(ValueCell::Map(HashMap::new()));
        let cloned = {
            let mut m = map.borrow_mut();
            m.as_mut_map()
            .unwrap()
            .insert(Value::U8(10).into(), map.clone());
            m.clone()
        };

        let mut inner_map: HashMap<ValueCell, SubValue> = HashMap::new();
        inner_map.insert(cloned, Value::U8(10).into());
    }

    #[test]
    fn test_stackoverflow_recursive_hash() {
        // Create a map that contains a map that contains a map...
        let mut map = ValueCell::Map(HashMap::new());
        for _ in 0..28000 {
            let mut inner_map = HashMap::new();
            inner_map.insert(Value::U8(10).into(), map.into());
            map = ValueCell::Map(inner_map);
        }
    }
}