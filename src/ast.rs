use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::idl;

pub enum ValidationError {
    DuplicateIdentifier(String),
}

#[derive(Default)]
pub struct Document {
    types: HashMap<String, Rc<Type>>,
    endpoints: HashMap<String, Rc<Endpoint>>,
    services: HashMap<String, Rc<Service>>,
}

pub enum Type {
    // builtin types
    Boolean,
    Integer,
    Float,
    String,
    UUID,
    Date,
    Time,
    DateTime,
    //
    Struct(Rc<Struct>),
    Array(Rc<Type>),
    Map(Rc<Type>, Rc<Type>),
    // named
    Unresolved(String),
}

pub struct Struct {
    name: String,
    fields: Vec<Rc<Field>>,
    field_by_name: HashMap<String, Rc<Field>>,
}

pub struct Field {
    name: String,
    type_: Rc<Type>,
    required: bool,
    // FIXME add options
}

pub struct Array {
    length: Range,
    item_type: Rc<Type>,
}

pub struct Map {
    length: Range,
    key_type: Rc<Type>,
    value_type: Rc<Type>,
}

pub struct Range {
    start: Option<i32>,
    end: Option<i32>,
}

pub struct Service {
    endpoints: Vec<Endpoint>,
}

pub struct Endpoint {
    name: String,
    in_: Rc<Type>,
    out: Rc<Type>,
    err: Rc<Type>,
}

impl Document {
    pub fn from_idl(idoc: &crate::idl::Document) -> Result<Self, ValidationError> {
        let mut doc = Self::default();
        for part in idoc.parts.iter() {
            match part {
                idl::NamespacePart::Struct(istruct) => {
                    if doc.types.contains_key(&istruct.name) {
                        return Err(ValidationError::DuplicateIdentifier(istruct.name.clone()));
                    }
                    doc.types.insert(
                        istruct.name.clone(),
                        Rc::new(Type::Struct(Rc::new(Struct::from_idl(&istruct)))),
                    );
                }
                // FIXME add support for more types
                _ => {}
            }
        }
        Ok(doc)
    }
}

impl Struct {
    pub fn from_idl(istruct: &idl::Struct) -> Self {
        let fields = istruct
            .fields
            .iter()
            .map(|ifield| {
                Rc::new(Field {
                    name: ifield.name.clone(),
                    type_: Rc::new(Type::from_idl(&ifield.type_)),
                    required: ifield.optional,
                    // FIXME add options
                    //options: ifield.options
                })
            })
            .collect();
        Struct {
            name: istruct.name.clone(),
            fields,
            // FIXME
            field_by_name: HashMap::default(),
        }
    }
}

impl Type {
    pub fn from_name(name: &str) -> Self {
        match name {
            "Boolean" => Self::Boolean,
            "Integer" => Self::Integer,
            "Float" => Self::Float,
            "String" => Self::String,
            "UUID" => Self::UUID,
            "Date" => Self::Date,
            "Time" => Self::Time,
            "DateTime" => Self::DateTime,
            name => Self::Unresolved(name.to_string()),
        }
    }
    pub fn from_idl(itype: &idl::Type) -> Self {
        match itype {
            idl::Type::Named(name) => Self::from_name(name),
            idl::Type::Array(item_type) => Self::Array(Rc::new(Self::from_name(item_type))),
            idl::Type::Map(key_type, value_type) => Self::Map(
                Rc::new(Self::from_name(key_type)),
                Rc::new(Self::from_name(value_type)),
            ),
        }
    }
}

impl From<idl::Type> for Type {
    fn from(itype: idl::Type) -> Self {
        match itype {
            // builtin types
            idl::Type::Named(name) => Type::from_name(name.as_str()),
            idl::Type::Array(item_type) => Type::Array(Rc::new(Type::from_name(item_type.as_str()))),
            idl::Type::Map(key_type, value_type) => Type::Map(
                Rc::new(Type::from_name(key_type.as_str())),
                Rc::new(Type::from_name(value_type.as_str())),
            ),
        }
    }
}
