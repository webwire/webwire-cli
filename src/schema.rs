use std::collections::HashMap;
use std::collections::hash_map::Entry as HashMapEntry;
use std::rc::{Rc, Weak};

use crate::common::FilePosition;
use crate::idl;

pub enum ValidationError {
    DuplicateIdentifier {
        position: FilePosition,
        identifier: String,
    }
}

pub enum Ref<T> {
    Resolved(Rc<T>),
    Unresolved(String),
}

#[derive(Default)]
pub struct Document {
    ns: Namespace,
}

#[derive(Default)]
pub struct Namespace {
    parent: Weak<Self>,
    types: HashMap<String, Type>,
    services: HashMap<String, Service>,
    namespaces: HashMap<String, Namespace>,
}

pub enum Type {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
    Array(TypeRef),
    Map(TypeRef, TypeRef),
}

pub enum TypeRef {
    // builtin types
    Boolean,
    Integer,
    Float,
    String,
    UUID,
    Date,
    Time,
    DateTime,
    // complex types
    Array(Box<Array>),
    Map(Box<Map>),
    // named
    UserDefined(Weak<Type>),
    Unresolved(String),
}

pub struct Enum {
    name: String,
    variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    name: String,
    value_type: Option<TypeRef>,
}

pub struct Struct {
    name: String,
    fields: Vec<Field>,
    field_by_name: HashMap<String, Rc<Field>>,
}

pub struct Field {
    name: String,
    type_: TypeRef,
    required: bool,
    // FIXME add options
}

pub struct Array {
    length: Range,
    item_type: TypeRef,
}

pub struct Map {
    length: Range,
    key_type: TypeRef,
    value_type: TypeRef,
}

pub struct Range {
    start: Option<i32>,
    end: Option<i32>,
}

pub struct Service {
    methods: Vec<Method>,
}

pub struct Method {
    name: String,
    input: Rc<Type>,
    output: Rc<Type>,
}

impl Document {
    pub fn from_idl(idoc: &crate::idl::Document) -> Result<Self, ValidationError> {
        Ok(Self {
            ns: Namespace::from_idl(&idoc.ns)?,
        })
    }
}

impl Namespace {
    pub fn from_idl(ins: &crate::idl::Namespace) -> Result<Self, ValidationError> {
        let mut ns = Self::default();
        ns.idl_convert(ins)?;
        ns.idl_resolve(ins)?;
        Ok(ns)
    }
    fn idl_convert(&mut self, ins: &crate::idl::Namespace) -> Result<(), ValidationError> {
        let mut names: HashMap<String, FilePosition> = HashMap::new();
        for ipart in ins.parts.iter() {
            match names.entry(ipart.name().to_owned()) {
                HashMapEntry::Occupied(entry) => {
                    return Err(ValidationError::DuplicateIdentifier {
                        position: entry.get().clone(),
                        identifier: ipart.name().to_owned(),
                    });
                }
                HashMapEntry::Vacant(entry) => {
                    entry.insert(ipart.position().clone());
                }
            }
            match ipart {
                idl::NamespacePart::Enum(ienum) => {
                    self.types.insert(
                        ipart.name().to_owned(),
                        Type::Enum(Enum::from_idl(&ienum)),
                    );
                }
                idl::NamespacePart::Struct(istruct) => {
                    self.types.insert(
                        ipart.name().to_owned(),
                        Type::Struct(Struct::from_idl(&istruct)),
                    );
                }
                idl::NamespacePart::Fieldset(ifieldset) => {
                    self.types.insert(
                        ipart.name().to_owned(),
                        Type::Fieldset(Fieldset::from_idl(&ifieldset)),
                    );
                }
                idl::NamespacePart::Service(iservice) => {
                    unimplemented!();
                }
                idl::NamespacePart::Namespace(inamespace) => {
                    let mut child_ns = Self::default();
                    child_ns.idl_convert(&inamespace)?;
                    self.namespaces.insert(
                        inamespace.name.to_owned(),
                        child_ns,
                    );
                }
            };
        }
        Ok(())
    }
    fn idl_resolve(&mut self, ins: &crate::idl::Namespace) -> Result<(), ValidationError> {
        // FIXME implement
        Ok(())
    }
}

impl Enum {
    pub fn from_idl(ienum: &idl::Enum) -> Self {
        let variants = ienum
            .variants
            .iter()
            .map(|ivariant| EnumVariant {
                name: ivariant.name.clone(),
                value_type: match &ivariant.value_type {
                    Some(itype) => Some(TypeRef::from_idl(&itype)),
                    None => None,
                },
            })
            .collect();
        Self {
            name: ienum.name.clone(),
            variants,
        }
    }
}

impl Struct {
    pub fn from_idl(istruct: &idl::Struct) -> Self {
        let fields = istruct
            .fields
            .iter()
            .map(|ifield| {
                Field {
                    name: ifield.name.clone(),
                    type_: TypeRef::from_idl(&ifield.type_),
                    required: ifield.optional,
                    // FIXME add options
                    //options: ifield.options
                }
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

impl TypeRef {
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
            idl::Type::Ref(idl::TypeRef {
                abs,
                ns,
                name,
                generics,
            }) => Self::from_name(name),
            idl::Type::Array(item_type) => Self::Array(Box::new(Array {
                item_type: Self::from_idl(item_type),
                length: Range { start: None, end: None }, // FIXME
            })),
            idl::Type::Map(key_type, value_type) => Self::Map(Box::new(Map {
                key_type: Self::from_idl(key_type),
                value_type: Self::from_idl(value_type),
                length: Range { start: None, end: None }, // FIXME
            })),
        }
    }
}

impl From<idl::Type> for TypeRef {
    fn from(itype: idl::Type) -> Self {
        TypeRef::from_idl(&itype)
    }
}

pub struct Fieldset {
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
