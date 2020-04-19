use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::idl;

pub enum ValidationError {
    DuplicateIdentifier(String),
}

pub enum Ref<T> {
    Resolved(Rc<T>),
    Unresolved(String),
}

#[derive(Default)]
pub struct Document {
    types: HashMap<String, Type>,
    endpoints: HashMap<String, Endpoint>,
    services: HashMap<String, Service>,
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
    Enum(Rc<Enum>),
    Struct(Rc<Struct>),
    Fieldset(Rc<Fieldset>),
    Array(Rc<Type>),
    Map(Rc<Type>, Rc<Type>),
    // named
    Unresolved(String),
}

pub struct Enum {
    name: String,
    values: Vec<String>,
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
        for ipart in idoc.parts.iter() {
            if doc.types.contains_key(ipart.name()) {
                return Err(ValidationError::DuplicateIdentifier(ipart.name().to_owned()));
            }
            match ipart {
                idl::NamespacePart::Enum(ienum) => {
                    doc.types.insert(
                        ipart.name().to_owned(),
                        Type::Enum(Rc::new(Enum::from_idl(&ienum))),
                    );
                }
                idl::NamespacePart::Struct(istruct) => {
                    doc.types.insert(
                        ipart.name().to_owned(),
                        Type::Struct(Rc::new(Struct::from_idl(&istruct))),
                    );
                }
                idl::NamespacePart::Fieldset(ifieldset) => {
                    doc.types.insert(
                        ipart.name().to_owned(),
                        Type::Fieldset(Rc::new(Fieldset::from_idl(&ifieldset))),
                    );
                }
                idl::NamespacePart::Operation(ioperation) => {

                }
                //idl::NamespacePart::Endpoint(iendpoint)
                //idl::NamespacePart::Service(iservice)
                //idl::NamespacePart::Namespace(inamespace)
            };

        }
        Ok(doc)
    }
}

impl Enum {
    pub fn from_idl(ienum: &idl::Enum) -> Self {
        let values = ienum.values.clone();
        Self {
            name: ienum.name.clone(),
            values,
        }
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
        Self {
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

struct Fieldset {
    name: String,
    r#struct: Ref<Struct>,
    fields: Vec<FieldsetField>,
}

type FieldsetField = idl::FieldsetField;

impl Fieldset {
    pub fn from_idl(ifieldset: &idl::Fieldset) -> Self {
        Self {
            name: ifieldset.name.clone(),
            r#struct: Ref::Unresolved(ifieldset.struct_name.clone()),
            fields: ifieldset.fields.clone(),
        }
    }
}