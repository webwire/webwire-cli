use crate::idl;

use super::namespace::Namespace;

/// Fully qualified type name
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FQTN {
    pub ns: Vec<String>,
    pub name: String,
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
            ns.extend_from_slice(&ityperef.ns);
            Self {
                ns,
                name: ityperef.name.clone(),
            }
        }
    }
}
