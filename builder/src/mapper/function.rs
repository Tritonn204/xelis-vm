use xelis_ast::{Expression, Signature};
use xelis_types::{IdentifierType, NoHashMap, Type};

use crate::BuilderError;

use super::Mapper;

/// Function structure
/// It contains the name of the function and its parameters
pub struct Function<'a> {
    pub name: &'a str,
    pub parameters: Vec<(&'a str, Type)>,
}

/// FunctionMapper is used to store the mapping between function signatures and their identifiers
/// So we can reduce the memory footprint of the VM by using an incremented id
pub struct FunctionMapper<'a> {
    mapper: Mapper<'a, Signature>,
    parent: Option<&'a FunctionMapper<'a>>,
    mappings: NoHashMap<Function<'a>>,
}

impl<'a> FunctionMapper<'a> {
    pub fn new() -> Self {
        Self {
            mapper: Mapper::new(),
            parent: None,
            mappings: NoHashMap::default()
        }
    }

    // Create a new Function Mapper
    pub fn with_parent(parent: &'a Self) -> Self {
        Self {
            mapper: Mapper::with_parent(&parent.mapper),
            parent: Some(parent),
            mappings: NoHashMap::default()
        }
    }

    // Get the identifier of a variable name
    pub fn get(&self, key: &Signature) -> Result<IdentifierType, BuilderError>
    {
        self.mapper.get(key)
    }

    // Register a function signature
    pub fn register(&mut self, name: &'a str, on_type: Option<Type>, parameters: Vec<(&'a str, Type)>) -> Result<IdentifierType, BuilderError> {
        let params: Vec<_> = parameters.iter().map(|(_, t)| t.clone()).collect();
        let signature = Signature::new(name.to_owned(), on_type, params);

        if self.mapper.has_variable(&signature) {
            return Err(BuilderError::SignatureAlreadyRegistered);
        }

        // Register the signature
        let id = self.mapper.register(signature)?;
        // Register the mappings
        self.mappings.insert(id.clone(), Function {
            name,
            parameters
        });

        Ok(id)
    }

    // Get a function mapp
    pub fn get_function(&self, id: &IdentifierType) -> Option<&Function<'a>> {
        self.mappings.get(id)
    }

    pub fn get_compatible(&self, key: Signature, expressions: &mut [Expression]) -> Result<IdentifierType, BuilderError> {
        // First check if we have the exact signature
        if let Ok(id) = self.mapper.get(&key) {
            return Ok(id);
        }

        // Lets find a compatible signature
        'main: for (signature, id) in self.mapper.mappings.iter().filter(|(s, _)| s.get_name() == key.get_name() && s.get_parameters().len() == key.get_parameters().len()) {            
            let on_type = match (signature.get_on_type(), key.get_on_type()) {
                (Some(s), Some(k)) => s.is_compatible_with(k),
                (None, None) => true,
                _ => false
            };

            if !on_type {
                continue;
            }

            let mut updated_expressions = Vec::new();
            for (i, (a, b)) in signature.get_parameters().iter().zip(key.get_parameters()).enumerate() {
                let mut cast_to_type = key.get_on_type()
                    .as_ref()
                    .map(Type::get_inner_type)
                    .filter(|t| b.is_castable_to(t));

                if cast_to_type.is_none() && !a.is_compatible_with(b) {
                    // If our parameter is castable to the signature parameter, cast it
                    if b.is_castable_to(a) {
                        cast_to_type = Some(a);
                    } else {
                        continue 'main;
                    }
                }

                // If cast is needed, cast it, if we fail, we continue to the next signature
                if let Some(a) = cast_to_type {
                    // We can only cast hardcoded values
                    if let Expression::Value(value) = &expressions[i] {
                        let cloned = value.clone();
                        let v = cloned.checked_cast_to_primitive_type(a)?;
                        updated_expressions.push(Expression::Value(v));
                        continue;
                    } else {
                        continue 'main;
                    }
                }
            }

            for (i, expr) in updated_expressions.into_iter().enumerate() {
                expressions[i] = expr;
            }

            return Ok(id.clone());
        }

        if let Some(parent) = self.parent {
            return parent.get_compatible(key, expressions);
        }

        Err(BuilderError::MappingNotFound)
    }
}