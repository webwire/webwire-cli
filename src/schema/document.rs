use super::errors::ValidationError;
use super::namespace::Namespace;

#[derive(Default)]
pub struct Document {
    pub ns: Namespace,
}

impl Document {
    pub fn from_idl<'a>(idocs: impl Iterator<Item = &'a crate::idl::Document>) -> Result<Self, ValidationError> {
        Ok(Self {
            ns: Namespace::from_idl(idocs.map(|idoc| &idoc.ns))?,
        })
    }
}
