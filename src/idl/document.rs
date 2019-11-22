use nom::{
    branch::alt,
    combinator::map,
    IResult,
    multi::separated_list,
    sequence::{preceded, terminated}
};

use crate::idl::common::{ws, ws1};
use crate::idl::endpoint::{Endpoint, parse_endpoint};
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
    Endpoint(Endpoint),
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
        map(parse_struct, DocumentPart::Struct),
        map(parse_endpoint, DocumentPart::Endpoint),
        map(parse_service, DocumentPart::Service),
    ))(input)
}

fn _parse_document(input: &str) -> IResult<&str, Document> {
    map(
        preceded(ws,
            terminated(
                separated_list(ws1, parse_document_part),
                ws
            )
        ),
        |parts| Document {
            parts: parts
        }
    )(input)
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    Nom(nom::Err<(&'a str, nom::error::ErrorKind)>),
    TrailingGarbage(String)
}

pub fn parse_document<'a>(input: &'a str) -> Result<Document, ParseError> {
    let result = _parse_document(input);
    match result {
        Ok(("", document)) => Ok(document),
        Ok((garbage, _)) => Err(ParseError::TrailingGarbage(garbage.to_string())),
        Err(error) => Err(ParseError::Nom(error))
    }
}

#[test]
fn test_parse_document() {
    use crate::idl::field_option::FieldOption;
    use crate::idl::r#struct::Field;
    use crate::idl::r#type::Type;
    use crate::idl::value::Value;
    let content = "
        struct Person {
            name: String (length=1..50),
            age: Integer
        }
        struct Group {
            name: String
        }
        endpoint ping()
        endpoint get_version() -> String
        service Pinger {
            ping,
            get_version
        }
    ";
    assert_eq!(
        parse_document(content),
        Ok(Document {
            parts: vec![
                DocumentPart::Struct(Struct {
                    name: "Person".to_string(),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            type_: Type::Named("String".to_string()),
                            optional: false,
                            options: vec![
                                FieldOption {
                                    name: "length".to_string(),
                                    value: Value::Range(Some(1), Some(50))
                                }
                            ],
                        },
                        Field {
                            name: "age".to_string(),
                            type_: Type::Named("Integer".to_string()),
                            optional: false,
                            options: vec![],
                        },
                    ],
                }),
                DocumentPart::Struct(Struct {
                    name: "Group".to_string(),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            type_: Type::Named("String".to_string()),
                            optional: false,
                            options: vec![],
                        },
                    ],
                }),
                DocumentPart::Endpoint(Endpoint {
                    name: "ping".to_string(),
                    request: None,
                    response: None,
                    error: None,
                }),
                DocumentPart::Endpoint(Endpoint {
                    name: "get_version".to_string(),
                    request: None,
                    response: Some("String".to_string()),
                    error: None,
                }),
                DocumentPart::Service(Service {
                    name: "Pinger".to_string(),
                    operations: vec![
                        "ping".to_string(),
                        "get_version".to_string(),
                    ],
                }),
            ]
        })
    )
}
