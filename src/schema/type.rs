use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;

use crate::common::FilePosition;
use crate::idl;

use super::errors::ValidationError;
use super::fieldset::Fieldset;
use super::fqtn::FQTN;
use super::namespace::Namespace;
use super::options::Range;
use super::r#enum::Enum;
use super::r#struct::Struct;
use super::typemap::TypeMap;

#[derive(Clone)]
pub enum Type {
    // builtin types
    None,
    Boolean,
    Integer,
    Float,
    String,
    UUID,
    Date,
    Time,
    DateTime,
    // complex types
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Array(Box<Array>),
    Map(Box<Map>),
    // named
    Ref(TypeRef),
    // builtin (user provided)
    Builtin(String),
}

#[derive(Clone)]
pub struct TypeRef {
    pub fqtn: FQTN,
    pub type_: Weak<RefCell<UserDefinedType>>,
    pub generics: Vec<Type>,
}

#[derive(Clone)]
pub struct Array {
    pub length: Range,
    pub item_type: Type,
}

#[derive(Clone)]
pub struct Map {
    pub length: Range,
    pub key_type: Type,
    pub value_type: Type,
}

pub enum UserDefinedType {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
}

impl Type {
    pub(crate) fn from_idl_ref(
        ityperef: &idl::TypeRef,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        // FIXME this should fail with an error when fqtn.ns is not empty
        match ityperef.name.as_str() {
            "None" => Self::None,
            "Boolean" => Self::Boolean,
            "Integer" => Self::Integer,
            "Float" => Self::Float,
            "String" => Self::String,
            "UUID" => Self::UUID,
            "Date" => Self::Date,
            "Time" => Self::Time,
            "DateTime" => Self::DateTime,
            "Option" => Self::Option(Box::new(Type::from_idl(
                &ityperef.generics[0],
                ns,
                &builtin_types,
            ))),
            "Result" => Self::Result(
                Box::new(Type::from_idl(&ityperef.generics[0], ns, &builtin_types)),
                Box::new(Type::from_idl(&ityperef.generics[1], ns, &builtin_types)),
            ),
            name => match builtin_types.get(name) {
                Some(value) => Self::Builtin(value.to_owned()),
                None => Self::Ref(TypeRef::from_idl(ityperef, ns, &builtin_types)),
            },
        }
    }
    pub(crate) fn from_idl(
        itype: &idl::Type,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        match itype {
            idl::Type::Ref(ityperef) => Self::from_idl_ref(&ityperef, &ns, &builtin_types),
            idl::Type::Array(item_type) => Self::Array(Box::new(Array {
                item_type: Self::from_idl(item_type, ns, &builtin_types),
                length: Range {
                    start: None,
                    end: None,
                }, // FIXME
            })),
            idl::Type::Map(key_type, value_type) => Self::Map(Box::new(Map {
                key_type: Self::from_idl(key_type, ns, &builtin_types),
                value_type: Self::from_idl(value_type, ns, &builtin_types),
                length: Range {
                    start: None,
                    end: None,
                }, // FIXME
            })),
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        match self {
            Self::None
            | Self::Boolean
            | Self::Integer
            | Self::Float
            | Self::String
            | Self::UUID
            | Self::Date
            | Self::Time
            | Self::DateTime => Ok(()),
            // complex types
            Self::Option(some) => some.resolve(type_map),
            Self::Result(ok, err) => {
                ok.resolve(type_map)?;
                err.resolve(type_map)?;
                Ok(())
            }
            Self::Array(array) => array.resolve(type_map),
            Self::Map(map) => map.resolve(type_map),
            // named
            Self::Ref(typeref) => typeref.resolve(type_map),
            // builtin (user defined)
            Self::Builtin(_) => Ok(()),
        }
    }
    /// Returns wether this type is scalar type or not.
    pub(crate) fn is_scalar(&self) -> bool {
        match self {
            Self::None
            | Self::Boolean
            | Self::Integer
            | Self::Float
            | Self::String
            | Self::UUID
            | Self::Date
            | Self::Time
            | Self::DateTime => true,
            Self::Option(type_) => type_.is_scalar(),
            Self::Result(_, _) => false,
            Self::Array(_) => false,
            Self::Map(_) => false,
            Self::Ref(_) => false,
            Self::Builtin(_) => true,
        }
    }
}

impl TypeRef {
    pub(crate) fn from_idl(
        ityperef: &idl::TypeRef,
        ns: &Namespace,
        builtin_types: &HashMap<String, String>,
    ) -> Self {
        Self {
            fqtn: FQTN::from_idl(ityperef, ns),
            type_: Weak::new(),
            generics: ityperef
                .generics
                .iter()
                .map(|itype| Type::from_idl(itype, ns, &builtin_types))
                .collect(),
        }
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        let type_ = type_map.get(&self.fqtn);
        let position = FilePosition { line: 0, column: 0 }; // FIXME
        self.type_ = match type_ {
            Some(type_) => {
                let ud_type = &*type_.upgrade().unwrap();
                if self.generics.len() != ud_type.borrow().generics().len() {
                    return Err(ValidationError::GenericsMissmatch {
                        fqtn: self.fqtn.clone(),
                        position,
                    });
                }
                type_
            }
            None => {
                return Err(ValidationError::NoSuchType {
                    fqtn: self.fqtn.clone(),
                    position,
                })
            }
        };
        Ok(())
    }
}

impl Array {
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        self.item_type.resolve(type_map)
    }
}

impl Map {
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        self.key_type.resolve(type_map)?;
        self.value_type.resolve(type_map)?;
        Ok(())
    }
}

impl UserDefinedType {
    pub fn fqtn(&self) -> &FQTN {
        match self {
            Self::Enum(t) => &t.fqtn,
            Self::Fieldset(t) => &t.fqtn,
            Self::Struct(t) => &t.fqtn,
        }
    }
    pub fn name(&self) -> &str {
        self.fqtn().name.as_str()
    }
    pub(crate) fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        match self {
            Self::Enum(t) => t.resolve(type_map),
            Self::Fieldset(t) => t.resolve(type_map),
            Self::Struct(t) => t.resolve(type_map),
        }
    }
    pub(crate) fn generics(&self) -> &Vec<String> {
        match self {
            Self::Enum(t) => &t.generics,
            Self::Fieldset(t) => &t.generics,
            Self::Struct(t) => &t.generics,
        }
    }
}

#[test]
fn test_schema_enum_extends() {
    let idl = r"
        enum Foo { Foo }
        enum Bar extends Foo { Bar }
    ";
    let idoc = crate::idl::parse_document(idl).unwrap();
    let idocs = vec![idoc];
    let builtin_types = HashMap::default();
    let doc = crate::schema::Document::from_idl(idocs.iter(), &builtin_types).unwrap();
    let foo = doc.ns.types.get("Bar").unwrap();
    let foo_ref = foo.borrow();
    match &*foo_ref {
        crate::schema::UserDefinedType::Enum(enum_) => {
            assert!(enum_.extends.is_some());
        }
        _ => unreachable!(),
    }
}
