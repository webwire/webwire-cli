use std::cell::RefCell;
use std::collections::hash_map::Entry as HashMapEntry;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::common::FilePosition;
use crate::idl;

pub enum ValidationError {
    DuplicateIdentifier {
        position: FilePosition,
        identifier: String,
    },
    NoSuchType {
        position: FilePosition,
        fqtn: FQTN,
    },
    GenericsMissmatch {
        position: FilePosition,
        fqtn: FQTN,
    },
}

#[derive(Default)]
pub struct Document {
    ns: Namespace,
}

#[derive(Default)]
pub struct Namespace {
    path: Vec<String>,
    types: HashMap<String, Rc<RefCell<UserDefinedType>>>,
    services: HashMap<String, Service>,
    namespaces: HashMap<String, Namespace>,
}

pub enum UserDefinedType {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
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
    // complex types
    Array(Box<Array>),
    Map(Box<Map>),
    // named
    Ref(TypeRef),
}

pub struct TypeRef {
    fqtn: FQTN,
    type_: Weak<RefCell<UserDefinedType>>,
    generics: Vec<Type>,
}

impl TypeRef {
    fn from_idl(ityperef: &idl::TypeRef, ns: &Namespace) -> Self {
        Self {
            fqtn: FQTN::from_idl(ityperef, ns),
            type_: Weak::new(),
            generics: ityperef
                .generics
                .iter()
                .map(|itype| Type::from_idl(itype, ns))
                .collect(),
        }
    }
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
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

/// Fully qualified type name
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct FQTN {
    ns: Vec<String>,
    name: String,
}

pub struct Enum {
    fqtn: FQTN,
    generics: Vec<String>,
    variants: Vec<EnumVariant>,
}

pub struct EnumVariant {
    name: String,
    value_type: Option<Type>,
}

pub struct Struct {
    fqtn: FQTN,
    generics: Vec<String>,
    fields: Vec<Field>,
    field_by_name: HashMap<String, Rc<Field>>,
}

pub struct Field {
    name: String,
    type_: Type,
    required: bool,
    // FIXME add options
}

pub struct Array {
    length: Range,
    item_type: Type,
}

pub struct Map {
    length: Range,
    key_type: Type,
    value_type: Type,
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
    input: Type,
    output: Type,
}

impl FQTN {
    pub fn new(name: &str, ns: &Namespace) -> Self {
        Self {
            ns: ns.path.clone(),
            name: name.to_owned(),
        }
    }
    pub fn from_idl(ityperef: &idl::TypeRef, ns: &Namespace) -> Self {
        if ityperef.abs {
            Self {
                ns: ityperef.ns.clone(),
                name: ityperef.name.clone(),
            }
        } else {
            let mut ns = ns.path.clone();
            // XXX why does `ns.extend` not work
            ns.extend_from_slice(&ityperef.ns);
            Self {
                ns,
                name: ityperef.name.clone(),
            }
        }
    }
}

impl Document {
    pub fn from_idl(idoc: &crate::idl::Document) -> Result<Self, ValidationError> {
        Ok(Self {
            ns: Namespace::from_idl(&idoc.ns)?,
        })
    }
}

#[derive(Default)]
pub struct TypeMap {
    map: HashMap<FQTN, Weak<RefCell<UserDefinedType>>>,
}

impl TypeMap {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    fn insert(&mut self, type_rc: &Rc<RefCell<UserDefinedType>>) {
        self.map
            .insert(type_rc.borrow().fqtn().clone(), Rc::downgrade(type_rc));
    }
    fn get(&self, fqtn: &FQTN) -> Option<Weak<RefCell<UserDefinedType>>> {
        match self.map.get(fqtn) {
            Some(type_rc) => Some(type_rc.clone()),
            None => None,
        }
    }
}

impl Namespace {
    pub fn from_idl(ins: &crate::idl::Namespace) -> Result<Self, ValidationError> {
        let mut ns = Self::default();
        let mut type_map = TypeMap::new();
        ns.idl_convert(ins, &mut type_map)?;
        ns.resolve(&type_map)?;
        Ok(ns)
    }
    fn add_type(&mut self, type_: UserDefinedType, type_map: &mut TypeMap) {
        let type_rc = Rc::new(RefCell::new(type_));
        type_map.insert(&type_rc);
        let name = type_rc.borrow().name().to_owned();
        self.types.insert(name, type_rc);
    }
    fn idl_convert(
        &mut self,
        ins: &crate::idl::Namespace,
        type_map: &mut TypeMap,
    ) -> Result<(), ValidationError> {
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
                    self.add_type(
                        UserDefinedType::Enum(Enum::from_idl(&ienum, self)),
                        type_map,
                    );
                }
                idl::NamespacePart::Struct(istruct) => {
                    self.add_type(
                        UserDefinedType::Struct(Struct::from_idl(&istruct, self)),
                        type_map,
                    );
                }
                idl::NamespacePart::Fieldset(ifieldset) => {
                    self.add_type(
                        UserDefinedType::Fieldset(Fieldset::from_idl(&ifieldset, self)),
                        type_map,
                    );
                }
                idl::NamespacePart::Service(_) => {
                    // This is done in the next step. Since services do not
                    // define any types we can ignore the merging and just
                    // delay processing of the service to the resolve step.
                }
                idl::NamespacePart::Namespace(inamespace) => {
                    let mut child_ns = Self::default();
                    child_ns.path = self.path.clone();
                    child_ns.path.push(ipart.name().to_owned());
                    child_ns.idl_convert(&inamespace, type_map)?;
                    self.namespaces.insert(inamespace.name.to_owned(), child_ns);
                }
            };
        }
        Ok(())
    }
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for type_rc in self.types.values() {
            type_rc.borrow_mut().resolve(type_map)?;
        }
        Ok(())
    }
}

impl Enum {
    pub fn from_idl(ienum: &idl::Enum, ns: &Namespace) -> Self {
        let variants = ienum
            .variants
            .iter()
            .map(|ivariant| EnumVariant {
                name: ivariant.name.clone(),
                value_type: match &ivariant.value_type {
                    Some(itype) => Some(Type::from_idl(itype, &ns)),
                    None => None,
                },
            })
            .collect();
        Self {
            fqtn: FQTN::new(&ienum.name, ns),
            generics: ienum.generics.clone(),
            variants,
        }
    }
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for variant in self.variants.iter_mut() {
            if let Some(typeref) = &mut variant.value_type {
                typeref.resolve(type_map)?;
            }
        }
        Ok(())
    }
}

impl Struct {
    pub fn from_idl(istruct: &idl::Struct, ns: &Namespace) -> Self {
        let fields = istruct
            .fields
            .iter()
            .map(|ifield| {
                Field {
                    name: ifield.name.clone(),
                    type_: Type::from_idl(&ifield.type_, ns),
                    required: ifield.optional,
                    // FIXME add options
                    //options: ifield.options
                }
            })
            .collect();
        Self {
            fqtn: FQTN::new(&istruct.name, ns),
            generics: istruct.generics.clone(),
            fields,
            field_by_name: HashMap::default(),
        }
    }
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for field in self.fields.iter_mut() {
            field.type_.resolve(type_map)?;
        }
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
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        match self {
            Self::Enum(t) => t.resolve(type_map),
            Self::Fieldset(t) => t.resolve(type_map),
            Self::Struct(t) => t.resolve(type_map),
        }
    }
    fn generics(&self) -> &Vec<String> {
        match self {
            Self::Enum(t) => &t.generics,
            Self::Fieldset(t) => &t.generics,
            Self::Struct(t) => &t.generics,
        }
    }
}

impl Type {
    pub fn from_idl_ref(ityperef: &idl::TypeRef, ns: &Namespace) -> Self {
        // FIXME this should fail with an error when fqtn.ns is not empty
        match ityperef.name.as_str() {
            "Boolean" => Self::Boolean,
            "Integer" => Self::Integer,
            "Float" => Self::Float,
            "String" => Self::String,
            "UUID" => Self::UUID,
            "Date" => Self::Date,
            "Time" => Self::Time,
            "DateTime" => Self::DateTime,
            _ => Self::Ref(TypeRef::from_idl(ityperef, ns)),
        }
    }
    pub fn from_idl(itype: &idl::Type, ns: &Namespace) -> Self {
        match itype {
            idl::Type::Ref(ityperef) => Self::from_idl_ref(&ityperef, &ns),
            idl::Type::Array(item_type) => Self::Array(Box::new(Array {
                item_type: Self::from_idl(item_type, ns),
                length: Range {
                    start: None,
                    end: None,
                }, // FIXME
            })),
            idl::Type::Map(key_type, value_type) => Self::Map(Box::new(Map {
                key_type: Self::from_idl(key_type, ns),
                value_type: Self::from_idl(value_type, ns),
                length: Range {
                    start: None,
                    end: None,
                }, // FIXME
            })),
        }
    }
    pub fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        match self {
            Type::Ref(typeref) => typeref.resolve(type_map),
            _ => Ok(()),
        }
    }
}

pub struct Fieldset {
    fqtn: FQTN,
    generics: Vec<String>,
    r#struct: TypeRef,
    fields: Vec<FieldsetField>,
}

type FieldsetField = idl::FieldsetField;

impl Fieldset {
    fn from_idl(ifieldset: &idl::Fieldset, ns: &Namespace) -> Self {
        Self {
            fqtn: FQTN::new(&ifieldset.name, ns),
            generics: ifieldset.generics.clone(),
            r#struct: TypeRef::from_idl(&ifieldset.r#struct, ns),
            fields: ifieldset.fields.clone(),
        }
    }
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        self.r#struct.resolve(type_map)?;
        // FIXME fields need to be resolved, too.
        Ok(())
    }
}
