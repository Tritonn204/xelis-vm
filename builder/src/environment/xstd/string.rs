use xelis_environment::Context;
use xelis_types::{Type, Value, ValueCell, ValuePointer};
use super::{
    FnInstance,
    FnParams,
    FnReturnType,
    EnvironmentBuilder
};

pub fn register(env: &mut EnvironmentBuilder) {
    // String
    env.register_native_function("len", Some(Type::String), vec![], len, 1, Some(Type::U32));
    env.register_native_function("trim", Some(Type::String), vec![], trim, 1, Some(Type::String));
    env.register_native_function("contains", Some(Type::String), vec![("value", Type::String)], contains, 1, Some(Type::Bool));
    env.register_native_function("contains_ignore_case", Some(Type::String), vec![("value", Type::String)], contains_ignore_case, 1, Some(Type::Bool));
    env.register_native_function("to_uppercase", Some(Type::String), vec![], to_uppercase, 1, Some(Type::String));
    env.register_native_function("to_lowercase", Some(Type::String), vec![], to_lowercase, 1, Some(Type::String));
    env.register_native_function("to_bytes", Some(Type::String), vec![], to_bytes, 5, Some(Type::Array(Box::new(Type::U8))));
    env.register_native_function("index_of", Some(Type::String), vec![("value", Type::String)], index_of, 3, Some(Type::Optional(Box::new(Type::U32))));
    env.register_native_function("last_index_of", Some(Type::String), vec![("value", Type::String)], last_index_of, 3, Some(Type::Optional(Box::new(Type::U32))));
    env.register_native_function("replace", Some(Type::String), vec![("from", Type::String), ("to", Type::String)], replace, 5, Some(Type::String));
    env.register_native_function("starts_with", Some(Type::String), vec![("value", Type::String)], starts_with, 3, Some(Type::Bool));
    env.register_native_function("ends_with", Some(Type::String), vec![("value", Type::String)], ends_with, 3, Some(Type::Bool));
    env.register_native_function("split", Some(Type::String), vec![("at", Type::String)], split, 5, Some(Type::Array(Box::new(Type::String))));
    env.register_native_function("char_at", Some(Type::String), vec![("index", Type::U32)], char_at, 1, Some(Type::Optional(Box::new(Type::String))));

    env.register_native_function("is_empty", Some(Type::String), vec![], is_empty, 1, Some(Type::Bool));
    env.register_native_function("matches", Some(Type::String), vec![("pattern", Type::String)], string_matches, 50, Some(Type::Array(Box::new(Type::String))));
    env.register_native_function("substring", Some(Type::String), vec![("value", Type::U32)], string_substring, 3, Some(Type::Optional(Box::new(Type::String))));
    env.register_native_function("substring", Some(Type::String), vec![("value", Type::U32), ("value", Type::U32)], string_substring_range, 3, Some(Type::Optional(Box::new(Type::String))));
}

fn len(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    Ok(Some(Value::U32(s.len() as u32).into()))
}

fn trim(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s = zelf?.as_string()?.trim().to_string();
    Ok(Some(Value::String(s).into()))
}

fn contains(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    let s: &String = zelf?.as_string()?;
    Ok(Some(Value::Boolean(s.contains(value)).into()))
}

fn contains_ignore_case(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let handle = parameters[0].handle();
    let value = handle.as_string()?.to_lowercase();
    let s: String = zelf?.as_string()?.to_lowercase();
    Ok(Some(Value::Boolean(s.contains(&value)).into()))
}

fn to_uppercase(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: String = zelf?.as_string()?.to_uppercase();
    Ok(Some(Value::String(s).into()))
}

fn to_lowercase(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: String = zelf?.as_string()?.to_lowercase();
    Ok(Some(Value::String(s).into()))
}

fn to_bytes(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;

    let mut bytes = Vec::new();
    for b in s.as_bytes() {
        bytes.push(ValuePointer::owned(Value::U8(*b).into()));
    }

    Ok(Some(ValueCell::Array(bytes)))
}

fn index_of(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    if let Some(index) = s.find(value) {
        let inner = ValuePointer::owned(Value::U32(index as u32).into());
        Ok(Some(ValueCell::Optional(Some(inner))))
    } else {
        Ok(Some(ValueCell::Optional(None)))
    }
}

fn last_index_of(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    if let Some(index) = s.rfind(value) {
        let inner = ValuePointer::owned(Value::U32(index as u32).into());
        Ok(Some(ValueCell::Optional(Some(inner))))
    } else {
        Ok(Some(ValueCell::Optional(None)))
    }
}

fn replace(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle1 = parameters[0].handle();
    let handle2 = parameters[1].handle();
    let old = handle1.as_string()?;
    let new = handle2.as_string()?;
    let s = s.replace(old, new);
    Ok(Some(Value::String(s).into()))
}

fn starts_with(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    Ok(Some(Value::Boolean(s.starts_with(value)).into()))
}

fn ends_with(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    Ok(Some(Value::Boolean(s.ends_with(value)).into()))
}

fn split(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    let values = s.split(value)
        .map(|s| ValuePointer::owned(Value::String(s.to_string()).into()))
        .collect();

    Ok(Some(ValueCell::Array(values)))
}

fn char_at(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let index =  parameters[0].handle().as_u32()? as usize;
    let s: &String = zelf?.as_string()?;
    if let Some(c) = s.chars().nth(index) {
        let inner = ValuePointer::owned(Value::String(c.to_string()).into());
        Ok(Some(ValueCell::Optional(Some(inner))))
    } else {
        Ok(Some(ValueCell::Optional(None)))
    }
}

fn is_empty(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    Ok(Some(Value::Boolean(s.is_empty()).into()))
}

fn string_matches(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let handle = parameters[0].handle();
    let value = handle.as_string()?;
    let m = s.matches(value);
    Ok(Some(ValueCell::Array(m.map(|s| ValuePointer::owned(Value::String(s.to_string()).into())).collect())))
}

fn string_substring(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let start = parameters[0].handle().as_u32()? as usize;
    if let Some(s) = s.get(start..) {
        let inner = ValuePointer::owned(Value::String(s.to_owned()).into());
        Ok(Some(ValueCell::Optional(Some(inner))))
    } else {
        Ok(Some(ValueCell::Optional(None)))
    }
}

fn string_substring_range(zelf: FnInstance, parameters: FnParams, _: &mut Context) -> FnReturnType {
    let s: &String = zelf?.as_string()?;
    let start = parameters[0].handle().as_u32()? as usize;
    let end = parameters[1].handle().as_u32()? as usize;

    if let Some(s) = s.get(start..end) {
        let inner = ValuePointer::owned(Value::String(s.to_owned()).into());
        Ok(Some(ValueCell::Optional(Some(inner))))
    } else {
        Ok(Some(ValueCell::Optional(None)))
    }
}