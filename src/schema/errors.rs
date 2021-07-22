use std::error::Error;
use std::fmt;

use crate::common::FilePosition;

use super::fqtn::FQTN;

#[derive(Debug)]
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
    FieldsetExtendsNonStruct {
        position: FilePosition,
        fieldset: FQTN,
        r#struct: FQTN,
    },
    NoSuchField {
        position: FilePosition,
        fieldset: FQTN,
        r#struct: FQTN,
        field: String,
    },
    EnumExtendsNonEnum {
        position: FilePosition,
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
