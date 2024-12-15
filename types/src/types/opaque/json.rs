use std::{collections::HashMap, sync::RwLock};
use lazy_static::lazy_static;
use serde_json::Value;
use super::OpaqueWrapper;

pub type DeserializeFn = dyn Fn(Value) -> Result<OpaqueWrapper, anyhow::Error> + Send + Sync;

lazy_static! {
    pub static ref JSON_REGISTRY: RwLock<HashMap<&'static str, Box<DeserializeFn>>> = RwLock::new(HashMap::new());
}

#[macro_export]
macro_rules! register_opaque {
    ($name:expr, $type:ty) => {
        impl $crate::types::opaque::JSONHelper for $type {
            fn get_type_name(&self) -> &'static str {
                $name
            }
    
            fn serialize_json(&self) -> Result<serde_json::Value, anyhow::Error> {
                Ok(serde_json::to_value(&self)?)
            }
        }

        $crate::types::opaque::JSON_REGISTRY.write()
            .unwrap()
            .insert($name, Box::new(|value| {
                use anyhow::Context;

                let value: $type = serde_json::from_value(value)
                    .context("Failed to deserialize JSON")?;

                Ok($crate::types::opaque::OpaqueWrapper::new(value))
            }));
    };
}

pub trait JSONHelper {
    // used to identify the type in the JSON
    // It must be unique across all types
    // and match the string used in `register_json!`
    fn get_type_name(&self) -> &'static str;

    fn serialize_json(&self) -> Result<Value, anyhow::Error>;
}