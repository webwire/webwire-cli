use crate::idl::namespace::{parse_namespace_content, NamespacePart};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub parts: Vec<NamespacePart>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    Nom(nom::Err<(&'a str, nom::error::ErrorKind)>),
    TrailingGarbage(String),
}

pub fn parse_document<'a>(input: &'a str) -> Result<Document, ParseError> {
    let result = parse_namespace_content(input);
    match result {
        Ok(("", parts)) => Ok(Document { parts: parts }),
        Ok((garbage, _)) => Err(ParseError::TrailingGarbage(garbage.to_string())),
        Err(error) => Err(ParseError::Nom(error)),
    }
}

#[test]
fn test_parse_document() {
    use crate::idl::endpoint::Endpoint;
    use crate::idl::field_option::FieldOption;
    use crate::idl::namespace::NamespacePart;
    use crate::idl::r#struct::{Field, Struct};
    use crate::idl::r#type::Type;
    use crate::idl::service::{Service, ServiceEndpoint};
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
            in ping,
            inout get_version
        }
    ";
    assert_eq!(
        parse_document(content),
        Ok(Document {
            parts: vec![
                NamespacePart::Struct(Struct {
                    name: "Person".to_string(),
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            type_: Type::Named("String".to_string()),
                            optional: false,
                            options: vec![FieldOption {
                                name: "length".to_string(),
                                value: Value::Range(Some(1), Some(50))
                            }],
                        },
                        Field {
                            name: "age".to_string(),
                            type_: Type::Named("Integer".to_string()),
                            optional: false,
                            options: vec![],
                        },
                    ],
                }),
                NamespacePart::Struct(Struct {
                    name: "Group".to_string(),
                    fields: vec![Field {
                        name: "name".to_string(),
                        type_: Type::Named("String".to_string()),
                        optional: false,
                        options: vec![],
                    },],
                }),
                NamespacePart::Endpoint(Endpoint {
                    name: "ping".to_string(),
                    request: None,
                    response: None,
                    error: None,
                }),
                NamespacePart::Endpoint(Endpoint {
                    name: "get_version".to_string(),
                    request: None,
                    response: Some("String".to_string()),
                    error: None,
                }),
                NamespacePart::Service(Service {
                    name: "Pinger".to_string(),
                    endpoints: vec![
                        ServiceEndpoint {
                            name: "ping".to_string(),
                            in_: true,
                            out: false
                        },
                        ServiceEndpoint {
                            name: "get_version".to_string(),
                            in_: true,
                            out: true
                        }
                    ],
                }),
            ]
        })
    )
}
