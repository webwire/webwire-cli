use std::error::Error;
use std::fmt;

use crate::common::FilePosition;

use super::fqtn::FQTN;

#[derive(Debug)]
pub struct ValidationError {
    pub position: FilePosition,
    pub cause: Box<ValidationErrorCause>,
}

#[derive(Debug)]
pub enum ValidationErrorCause {
    DuplicateIdentifier {
        identifier: String,
    },
    NoSuchType {
        fqtn: FQTN,
    },
    GenericsMissmatch {
        fqtn: FQTN,
    },
    FieldsetExtendsNonStruct {
        fieldset: FQTN,
        r#struct: FQTN,
    },
    NoSuchField {
        fieldset: FQTN,
        r#struct: FQTN,
        field: String,
    },
    EnumExtendsNonEnum {
        r#enum: FQTN,
        extends: FQTN,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // FIXME Replace this by a proper implementation of Display
        write!(f, "{:?}", self)
    }
}

impl Error for ValidationError {}
