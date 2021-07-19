use std::collections::HashMap;

use crate::idl;

use super::errors::ValidationError;
use super::namespace::Namespace;
use super::r#type::Type;
use super::typemap::TypeMap;

pub struct Service {
    pub name: String,
    pub methods: Vec<Method>,
}

pub struct Method {
    pub name: String,
    pub input: Option<Type>,
    pub output: Option<Type>,
}

impl Service {
    pub(crate) fn from_idl(
        iservice: &idl::Service,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        Self {
            name: iservice.name.clone(),
            methods: iservice
                .methods
                .iter()
                .map(|imethod| Method {
                    name: imethod.name.clone(),
                    input: if let Some(x) = &imethod.input {
                        Some(Type::from_idl(x, ns, &builtin_types))
                    } else {
                        None
                    },
                    output: if let Some(x) = &imethod.output {
                        Some(Type::from_idl(x, ns, &builtin_types))
                    } else {
                        None
                    },
                })
                .collect(),
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for method in self.methods.iter_mut() {
            if let Some(input) = &mut method.input {
                input.resolve(type_map)?;
            }
            if let Some(output) = &mut method.output {
                output.resolve(type_map)?;
            }
        }
        Ok(())
    }
}
