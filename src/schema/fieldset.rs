use std::collections::HashMap;

use crate::common::FilePosition;
use crate::idl;

use super::errors::{ValidationError, ValidationErrorCause};
use super::fqtn::FQTN;
use super::namespace::Namespace;
use super::r#struct::Field;
use super::r#type::TypeRef;
use super::typemap::TypeMap;

pub struct Fieldset {
    pub fqtn: FQTN,
    pub generics: Vec<String>,
    pub r#struct: TypeRef,
    pub fields: Vec<FieldsetField>,
}

pub struct FieldsetField {
    pub name: String,
    pub optional: bool,
    pub field: Option<Field>,
}

impl Fieldset {
    pub(crate) fn from_idl(
        ifieldset: &idl::Fieldset,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        Self {
            fqtn: FQTN::new(&ifieldset.name, ns),
            generics: ifieldset.generics.clone(),
            r#struct: TypeRef::from_idl(&ifieldset.r#struct, ns, builtin_types),
            fields: ifieldset
                .fields
                .iter()
                .map(|ifield| FieldsetField {
                    name: ifield.name.clone(),
                    optional: ifield.optional,
                    field: None,
                })
                .collect(),
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        self.r#struct.resolve(type_map)?;
        if let TypeRef::Struct(struct_) = &self.r#struct {
            let struct_rc = struct_.struct_.upgrade().unwrap();
            let struct_borrow = struct_rc.borrow();
            let field_map = struct_borrow
                .fields
                .iter()
                .map(|f| (f.name.clone(), f))
                .collect::<HashMap<_, _>>();
            for field in self.fields.iter_mut() {
                if let Some(&struct_field) = field_map.get(&field.name) {
                    field.field.replace(struct_field.clone());
                } else {
                    return Err(ValidationError {
                        position: FilePosition { line: 0, column: 0 },
                        cause: Box::new(ValidationErrorCause::NoSuchField {
                            fieldset: self.fqtn.clone(),
                            r#struct: struct_borrow.fqtn.clone(),
                            field: field.name.clone(),
                        })
                    });
                }
            }
        } else {
            return Err(ValidationError {
                position: FilePosition { line: 0, column: 0 },
                cause: Box::new(ValidationErrorCause::FieldsetExtendsNonStruct {
                    fieldset: self.fqtn.clone(),
                    r#struct: self.r#struct.fqtn().clone(),
                })
            });
        }
        // FIXME fields need to be resolved, too.
        Ok(())
    }
}
