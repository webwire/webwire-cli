use std::collections::HashMap;

use crate::idl;

use super::errors::ValidationError;
use super::fqtn::FQTN;
use super::namespace::Namespace;
use super::r#type::Type;
use super::typemap::TypeMap;

pub struct Enum {
    pub fqtn: FQTN,
    pub generics: Vec<String>,
    pub variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    pub name: String,
    pub value_type: Option<Type>,
}

impl Enum {
    pub(crate) fn from_idl(
        ienum: &idl::Enum,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        let variants = ienum
            .variants
            .iter()
            .map(|ivariant| EnumVariant {
                name: ivariant.name.clone(),
                value_type: ivariant
                    .value_type
                    .as_ref()
                    .map(|itype| Type::from_idl(itype, &ns, &builtin_types)),
            })
            .collect();
        Self {
            fqtn: FQTN::new(&ienum.name, ns),
            generics: ienum.generics.clone(),
            variants,
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for variant in self.variants.iter_mut() {
            if let Some(typeref) = &mut variant.value_type {
                typeref.resolve(type_map)?;
            }
        }
        Ok(())
    }
}
