use nom::{
    IResult,
    branch::alt,
    bytes::complete::{take_while, take_while1},
    combinator::map,
    multi::separated_list,
};

use crate::idl::r#enum::{Enum, parse_enum};
use crate::idl::fieldset::{Fieldset, parse_fieldset};
use crate::idl::operation::{Operation, parse_operation};
use crate::idl::service::{Service, parse_service};
use crate::idl::r#struct::{Struct, parse_struct};

#[derive(Debug, PartialEq)]
pub enum DocumentPart {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
    Operation(Operation),
    Service(Service)
}

#[derive(Debug, PartialEq)]
pub struct Document {
    pub parts: Vec<DocumentPart>
}

fn parse_document_part(input: &str) -> IResult<&str, DocumentPart> {
    alt((
        map(parse_enum, DocumentPart::Enum),
        map(parse_fieldset, DocumentPart::Fieldset),
        map(parse_operation, DocumentPart::Operation),
        map(parse_service, DocumentPart::Service),
        map(parse_struct, DocumentPart::Struct),
    ))(input)
}

pub fn parse_document(input: &str) -> IResult<&str, Document> {
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, parts) = separated_list(take_while1(char::is_whitespace), parse_document_part)(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    // FIXME fail if there is remaining input
    Ok((input, Document {
        parts: parts
    }))
}

#[test]
fn test_parse_document() {
    use crate::idl::r#struct::Field as StructField;
    use crate::idl::r#type::Type;
    let content = "
        struct Person {
            name: String,
            age: Integer
        }
        struct Group {
            name: String
        }
        service Pinger {
            ping,
            get_version
        }
    ";
    assert_eq!(
        parse_document(content),
        Ok(("", Document {
            parts: vec![
                DocumentPart::Struct(Struct {
                    name: "Person".to_string(),
                    fields: vec![
                        StructField { name: "name".to_string(), type_: Type::Named("String".to_string()), optional: false },
                        StructField { name: "age".to_string(), type_: Type::Named("Integer".to_string()), optional: false },
                    ],
                }),
                DocumentPart::Struct(Struct {
                    name: "Group".to_string(),
                    fields: vec![
                        StructField { name: "name".to_string(), type_: Type::Named("String".to_string()), optional: false },
                    ],
                }),
                DocumentPart::Service(Service {
                    name: "Pinger".to_string(),
                    operations: vec![
                        "ping".to_string(),
                        "get_version".to_string(),
                    ],
                }),
            ]
        }))
    )
}
