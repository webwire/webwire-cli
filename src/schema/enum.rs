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
    pub fn extends_enum(&self) -> Option<Rc<RefCell<Enum>>> {
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
    match foo {
        crate::schema::UserDefinedType::Enum(enum_) => {
            assert!(enum_.borrow().extends.is_some());
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_schema_enum_extends_different_namespace() {
    let idl = r"
        enum Foo { Foo }
        namespace bar {
            enum Bar extends ::Foo { Bar }
        }
        namespace baz {
            enum Baz extends ::bar::Bar { Baz }
        }
    ";
    let idoc = crate::idl::parse_document(idl).unwrap();
    let idocs = vec![idoc];
    let builtin_types = HashMap::default();
    let doc = crate::schema::Document::from_idl(idocs.iter(), &builtin_types).unwrap();
    let bar_ns = doc.ns.namespaces.get("bar").unwrap();
    let bar_type = bar_ns.types.get("Bar").unwrap();
    match bar_type {
        crate::schema::UserDefinedType::Enum(enum_) => {
            assert!(enum_.borrow().extends.is_some());
        }
        _ => unreachable!(),
    }
    let baz_ns = doc.ns.namespaces.get("baz").unwrap();
    let baz_type = baz_ns.types.get("Baz").unwrap();
    match baz_type {
        crate::schema::UserDefinedType::Enum(enum_) => {
            assert!(enum_.borrow().extends.is_some());
        }
        _ => unreachable!(),
    }
}
