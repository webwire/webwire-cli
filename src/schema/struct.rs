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
    pub required: bool,
    // FIXME add options
    pub position: FilePosition,
}

impl Struct {
    pub(crate) fn from_idl(istruct: &idl::Struct, ns: &Namespace) -> Self {
        let fields = istruct
            .fields
            .iter()
            .map(|ifield| Field::from_idl(ifield, ns))
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
    pub fn from_idl(ifield: &idl::Field, ns: &Namespace) -> Self {
        Field {
            name: ifield.name.clone(),
            type_: Type::from_idl(&ifield.type_, ns),
            required: ifield.optional,
            // FIXME add options
            //options: ifield.options
            position: ifield.position.clone(),
        }
    }
}
