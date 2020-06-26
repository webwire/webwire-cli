use crate::common::FilePosition;
use crate::idl::common::Span;
use crate::idl::namespace::{parse_namespace_content, Namespace};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub ns: Namespace,
}

#[derive(Debug, PartialEq)]
pub enum ParseError<'a> {
    Nom(nom::Err<(Span<'a>, nom::error::ErrorKind)>),
    TrailingGarbage(Span<'a>),
}

pub fn parse_document(input: Span) -> Result<Document, ParseError> {
    let result = parse_namespace_content(input);
    match result {
        Ok((span, parts)) if span.fragment() == &"" => Ok(Document {
            ns: Namespace {
                name: String::default(),
                position: FilePosition { line: 1, column: 1 },
                parts,
            },
        }),
        Ok((garbage, _)) => Err(ParseError::TrailingGarbage(garbage)),
        Err(error) => Err(ParseError::Nom(error)),
    }
}

#[test]
fn test_parse_document() {
    use crate::idl::field_option::FieldOption;
    use crate::idl::method::Method;
    use crate::idl::namespace::{Namespace, NamespacePart};
    use crate::idl::r#struct::{Field, Struct};
    use crate::idl::r#type::{Type, TypeRef};
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
            ns: Namespace {
                name: "".to_string(),
                position: FilePosition { line: 1, column: 1 },
                parts: vec![
                    NamespacePart::Struct(Struct {
                        name: "Person".to_string(),
                        position: FilePosition { line: 2, column: 9 },
                        generics: vec![],
                        fields: vec![
                            Field {
                                name: "name".to_string(),
                                position: FilePosition {
                                    line: 3,
                                    column: 13
                                },
                                type_: Type::Ref(TypeRef {
                                    abs: false,
                                    ns: vec![],
                                    name: "String".to_string(),
                                    generics: vec![]
                                }),
                                optional: false,
                                options: vec![FieldOption {
                                    name: "length".to_string(),
                                    value: Value::Range(Some(1), Some(50))
                                }],
                            },
                            Field {
                                name: "age".to_string(),
                                position: FilePosition {
                                    line: 4,
                                    column: 13
                                },
                                type_: Type::Ref(TypeRef {
                                    abs: false,
                                    name: "Integer".to_string(),
                                    ns: vec![],
                                    generics: vec![],
                                }),
                                optional: false,
                                options: vec![],
                            },
                        ],
                    }),
                    NamespacePart::Struct(Struct {
                        name: "Group".to_string(),
                        position: FilePosition { line: 6, column: 9 },
                        generics: vec![],
                        fields: vec![Field {
                            name: "name".to_string(),
                            position: FilePosition {
                                line: 7,
                                column: 13
                            },
                            type_: Type::Ref(TypeRef {
                                abs: false,
                                name: "String".to_string(),
                                ns: vec![],
                                generics: vec![]
                            }),
                            optional: false,
                            options: vec![],
                        }],
                    }),
                    NamespacePart::Service(Service {
                        name: "Pinger".to_string(),
                        position: FilePosition { line: 9, column: 9 },
                        methods: vec![
                            Method {
                                name: "ping".to_string(),
                                input: None,
                                output: None,
                            },
                            Method {
                                name: "get_version".to_string(),
                                input: None,
                                output: Some(Type::Ref(TypeRef {
                                    abs: false,
                                    ns: vec![],
                                    name: "String".to_string(),
                                    generics: vec![]
                                })),
                            },
                        ],
                    }),
                ],
            },
        })
    )
}
