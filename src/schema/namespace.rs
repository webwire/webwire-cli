use std::cell::RefCell;
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
    pub types: BTreeMap<String, Rc<RefCell<UserDefinedType>>>,
    pub services: BTreeMap<String, Service>,
    pub namespaces: BTreeMap<String, Namespace>,
}

impl Namespace {
    pub(crate) fn from_idl<'a>(
        inss: impl Iterator<Item = &'a crate::idl::Namespace>,
    ) -> Result<Self, ValidationError> {
        let mut ns = Self::default();
        let mut type_map = TypeMap::new();
        for ins in inss {
            ns.idl_convert(ins, &mut type_map)?;
        }
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
                idl::NamespacePart::Service(iservice) => {
                    self.services
                        .insert(iservice.name.clone(), Service::from_idl(iservice, self));
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
