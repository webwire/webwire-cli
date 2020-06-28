use super::errors::ValidationError;
use super::namespace::Namespace;

#[derive(Default)]
pub struct Document {
    pub ns: Namespace,
}

impl Document {
    pub fn from_idl(idoc: &crate::idl::Document) -> Result<Self, ValidationError> {
        Ok(Self {
            ns: Namespace::from_idl(&idoc.ns)?,
        })
    }
}
