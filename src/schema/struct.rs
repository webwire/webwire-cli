use std::collections::HashMap;

use crate::common::FilePosition;
use crate::idl;

use super::errors::ValidationError;
use super::fqtn::FQTN;
use super::namespace::Namespace;
use super::r#type::Type;
use super::typemap::TypeMap;

pub struct Struct {
    pub fqtn: FQTN,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
    pub position: FilePosition,
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub type_: Type,
    pub optional: bool,
    // FIXME add options
    pub length: (Option<i64>, Option<i64>),
    pub format: Option<String>,
    pub position: FilePosition,
}

impl Struct {
    pub(crate) fn from_idl(
        istruct: &idl::Struct,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        let fields = istruct
            .fields
            .iter()
            .map(|ifield| Field::from_idl(ifield, ns, builtin_types))
            .collect();
        Self {
            fqtn: FQTN::new(&istruct.name, ns),
            generics: istruct.generics.clone(),
            fields,
            position: istruct.position.clone(),
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for field in self.fields.iter_mut() {
            field.type_.resolve(type_map)?;
        }
        Ok(())
    }
}

impl Field {
    pub fn from_idl(
        ifield: &idl::Field,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        let mut length: (Option<i64>, Option<i64>) = (None, None);
        let mut format: Option<String> = None;
        for option in &ifield.options {
            match (option.name.as_str(), &option.value) {
                ("length", idl::Value::Range(min, max)) => length = (*min, *max),
                //("format", format) => format = Some(format),
                ("format", idl::Value::String(f)) => format = Some(f.clone()),
                (name, _) => panic!("Unsupported option: {}", name),
            }
        }
        Field {
            name: ifield.name.clone(),
            type_: Type::from_idl(&ifield.type_, ns, builtin_types),
            optional: ifield.optional,
            // FIXME add options
            //options: ifield.options
            length,
            format,
            position: ifield.position.clone(),
        }
    }
}
