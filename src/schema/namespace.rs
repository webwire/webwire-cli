use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::{btree_map::Entry as BTreeMapEntry, BTreeMap};
use std::rc::Rc;

use crate::common::FilePosition;
use crate::idl;

use super::errors::ValidationError;
use super::fieldset::Fieldset;
use super::r#enum::Enum;
use super::r#struct::Struct;
use super::r#type::UserDefinedType;
use super::service::Service;
use super::typemap::TypeMap;

#[derive(Default)]
pub struct Namespace {
    pub path: Vec<String>,
    pub types: BTreeMap<String, UserDefinedType>,
    pub services: BTreeMap<String, Service>,
    pub namespaces: BTreeMap<String, Namespace>,
}

impl Namespace {
    pub(crate) fn from_idl<'a>(
        inss: impl Iterator<Item = &'a crate::idl::Namespace>,
        builtin_types: &HashMap<String, String>,
    ) -> Result<Self, ValidationError> {
        let mut ns = Self::default();
        let mut type_map = TypeMap::new();
        for ins in inss {
            ns.idl_convert(ins, &mut type_map, &builtin_types)?;
        }
        ns.resolve(&type_map)?;
        Ok(ns)
    }
    fn add_type(&mut self, type_: UserDefinedType, type_map: &mut TypeMap) {
        type_map.insert(&type_);
        self.types.insert(type_.fqtn().name.to_owned(), type_);
    }
    fn idl_convert(
        &mut self,
        ins: &crate::idl::Namespace,
        type_map: &mut TypeMap,
        builtin_types: &HashMap<String, String>,
    ) -> Result<(), ValidationError> {
        let mut names: BTreeMap<String, FilePosition> = BTreeMap::new();
        for ipart in ins.parts.iter() {
            match names.entry(ipart.name().to_owned()) {
                BTreeMapEntry::Occupied(entry) => {
                    return Err(ValidationError::DuplicateIdentifier {
                        position: entry.get().clone(),
                        identifier: ipart.name().to_owned(),
                    });
                }
                BTreeMapEntry::Vacant(entry) => {
                    entry.insert(ipart.position().clone());
                }
            }
            match ipart {
                idl::NamespacePart::Enum(ienum) => {
                    self.add_type(
                        UserDefinedType::Enum(Rc::new(RefCell::new(Enum::from_idl(
                            &ienum,
                            self,
                            &builtin_types,
                        )))),
                        type_map,
                    );
                }
                idl::NamespacePart::Struct(istruct) => {
                    self.add_type(
                        UserDefinedType::Struct(Rc::new(RefCell::new(Struct::from_idl(
                            &istruct,
                            self,
                            &builtin_types,
                        )))),
                        type_map,
                    );
                }
                idl::NamespacePart::Fieldset(ifieldset) => {
                    self.add_type(
                        UserDefinedType::Fieldset(Rc::new(RefCell::new(Fieldset::from_idl(
                            &ifieldset,
                            self,
                            &builtin_types,
                        )))),
                        type_map,
                    );
                }
                idl::NamespacePart::Service(iservice) => {
                    self.services.insert(
                        iservice.name.clone(),
                        Service::from_idl(iservice, self, &builtin_types),
                    );
                    // This is done in the next step. Since services do not
                    // define any types we can ignore the merging and just
                    // delay processing of the service to the resolve step.
                }
                idl::NamespacePart::Namespace(inamespace) => {
                    let mut child_ns = Self {
                        path: self.path.clone(),
                        ..Default::default()
                    };
                    child_ns.path.push(ipart.name().to_owned());
                    child_ns.idl_convert(&inamespace, type_map, &builtin_types)?;
                    self.namespaces.insert(inamespace.name.to_owned(), child_ns);
                }
            };
        }
        Ok(())
    }
    fn resolve(&mut self, type_map: &TypeMap) -> Result<(), ValidationError> {
        for ud_type in self.types.values_mut() {
            ud_type.resolve(type_map)?;
        }
        for service in self.services.values_mut() {
            service.resolve(type_map)?;
        }
        for child_ns in self.namespaces.values_mut() {
            child_ns.resolve(type_map)?;
        }
        Ok(())
    }
    pub fn name(&self) -> &str {
        self.path.last().unwrap()
    }
}
