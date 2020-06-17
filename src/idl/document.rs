use crate::idl::common::Span;
use crate::idl::namespace::{parse_namespace_content, NamespacePart};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub parts: Vec<NamespacePart>,
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    Nom(nom::Err<(Span<'a>, nom::error::ErrorKind)>),
    TrailingGarbage(Span<'a>),
}

pub fn parse_document(input: Span) -> Result<Document, ParseError> {
    let result = parse_namespace_content(input);
    match result {
        Ok((span, parts)) if span.fragment() == &"" => Ok(Document { parts: parts }),
        Ok((garbage, _)) => Err(ParseError::TrailingGarbage(garbage)),
        Err(error) => Err(ParseError::Nom(error)),
    }
}

#[test]
fn test_parse_document() {
    use crate::idl::field_option::FieldOption;
    use crate::idl::method::Method;
    use crate::idl::namespace::NamespacePart;
    use crate::idl::r#struct::{Field, Struct};
    use crate::idl::r#type::Type;
    use crate::idl::service::Service;
    use crate::idl::value::Value;
    let content = "
        struct Person {
            name: String (length=1..50),
            age: Integer,
        }
        struct Group {
            name: String,
        }
        service Pinger {
            ping(),
            get_version() -> String,
        }
    ";
    assert_eq!(
        parse_document(Span::new(content)),
        Ok(Document {
            parts: vec![
                NamespacePart::Struct(Struct {
                    name: "Person".to_string(),
                    generics: vec![],
                    fields: vec![
                        Field {
                            name: "name".to_string(),
                            type_: Type::Named("String".to_string(), vec![]),
                            optional: false,
                            options: vec![FieldOption {
                                name: "length".to_string(),
                                value: Value::Range(Some(1), Some(50))
                            }],
                        },
                        Field {
                            name: "age".to_string(),
                            type_: Type::Named("Integer".to_string(), vec![]),
                            optional: false,
                            options: vec![],
                        },
                    ],
                }),
                NamespacePart::Struct(Struct {
                    name: "Group".to_string(),
                    generics: vec![],
                    fields: vec![Field {
                        name: "name".to_string(),
                        type_: Type::Named("String".to_string(), vec![]),
                        optional: false,
                        options: vec![],
                    },],
                }),
                NamespacePart::Service(Service {
                    name: "Pinger".to_string(),
                    methods: vec![
                        Method {
                            name: "ping".to_string(),
                            request: None,
                            response: None,
                        },
                        Method {
                            name: "get_version".to_string(),
                            request: None,
                            response: Some(Type::Named("String".to_string(), vec![])),
                        },
                    ],
                }),
            ]
        })
    )
}
