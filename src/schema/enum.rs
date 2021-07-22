use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::common::FilePosition;
use crate::idl;

use super::errors::ValidationError;
use super::fqtn::FQTN;
use super::namespace::Namespace;
use super::r#type::Type;
use super::typemap::TypeMap;
use super::TypeRef;

#[derive(Clone)]
pub struct Enum {
    pub fqtn: FQTN,
    pub generics: Vec<String>,
    pub extends: Option<TypeRef>,
    pub variants: Vec<EnumVariant>,
    pub all_variants: Vec<EnumVariant>,
}

#[derive(Clone)]
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
        let extends = ienum
            .extends
            .as_ref()
            .map(|itype| TypeRef::from_idl(itype, &ns, &builtin_types));
        Self {
            fqtn: FQTN::new(&ienum.name, ns),
            generics: ienum.generics.clone(),
            extends,
            variants,
            all_variants: Vec::new(),
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for variant in self.variants.iter_mut() {
            if let Some(typeref) = &mut variant.value_type {
                typeref.resolve(type_map)?;
            }
        }
        if let Some(extends) = &mut self.extends {
            extends.resolve(type_map)?;
        }
        self.all_variants.extend(self.resolve_extends()?);
        Ok(())
    }
    fn resolve_extends(&self) -> Result<Vec<EnumVariant>, ValidationError> {
        let mut variants = self.variants.clone();
        if let Some(extends) = &self.extends {
            if let TypeRef::Enum(extends_enum) = extends {
                variants.extend(
                    extends_enum
                        .enum_
                        .upgrade()
                        .unwrap()
                        .borrow()
                        .resolve_extends()?,
                );
            } else {
                return Err(ValidationError::EnumExtendsNonEnum {
                    position: FilePosition { line: 0, column: 0 },
                    r#enum: self.fqtn.clone(),
                    extends: extends.fqtn(),
                });
            }
        }
        Ok(variants)
    }
    pub fn extends(&self) -> Option<Rc<RefCell<Enum>>> {
        if let Some(extends) = &self.extends {
            if let TypeRef::Enum(extends) = extends {
                Some(extends.enum_.upgrade().unwrap())
            } else {
                None
            }
        } else {
            None
        }
    }
}
