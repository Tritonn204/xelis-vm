use xelis_types::{Type, Value};
use xelis_environment::{
    FnInstance,
    FnParams,
    FnReturnType,
    Context
};
use super::EnvironmentBuilder;

pub fn register(env: &mut EnvironmentBuilder) {
    env.register_native_function("is_none", Some(Type::Optional(Box::new(Type::T(0)))), vec![], is_none, 1, Some(Type::Bool));
    env.register_native_function("is_some", Some(Type::Optional(Box::new(Type::T(0)))), vec![], is_some, 1, Some(Type::Bool));
    env.register_native_function("unwrap", Some(Type::Optional(Box::new(Type::T(0)))), vec![], unwrap, 1, Some(Type::T(0)));
    env.register_native_function("unwrap_or", Some(Type::Optional(Box::new(Type::T(0)))), vec![Type::T(0)], unwrap_or, 1, Some(Type::T(0)));
}

fn is_none(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    Ok(Some(Value::Boolean(zelf?.as_optional(&Type::T(0))?.is_none())))
}

fn is_some(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    Ok(Some(Value::Boolean(zelf?.as_optional(&Type::T(0))?.is_some())))
}

fn unwrap(zelf: FnInstance, _: FnParams, _: &mut Context) -> FnReturnType {
    let mut opt = zelf?.take_from_optional(&Type::T(0))?;
    Ok(Some(opt.into_value()))
}

fn unwrap_or(zelf: FnInstance, mut parameters: FnParams, _: &mut Context) -> FnReturnType {
    let default = parameters.remove(0);
    let optional = zelf?.take_optional()?;
    match optional {
        Some(mut value) => Ok(Some(value.into_value())),
        None => Ok(Some(default.into_owned()))
    }
}