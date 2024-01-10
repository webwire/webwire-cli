use nom::{
    branch::alt,
    combinator::map,
    multi::separated_list0,
    sequence::{preceded, terminated},
    IResult,
};

use crate::common::FilePosition;
use crate::idl::common::Span;
use crate::idl::errors::ParseError;
use crate::idl::namespace::Namespace;

use super::{
    common::{ws, ws1},
    include::{parse_include, Include},
    namespace::parse_namespace_part,
    NamespacePart,
};

#[derive(Debug, PartialEq)]
pub struct Document {
    pub includes: Vec<Include>,
    pub ns: Namespace,
}

pub enum DocumentPart {
    Include(Include),
    NamespacePart(NamespacePart),
}

pub fn parse_document(input: &str) -> Result<Document, ParseError> {
    let span = Span::new(input);
    let result = parse_document_content(span);
    match result {
        Ok((span, parts)) if span.fragment() == &"" => {
            let mut includes: Vec<Include> = Vec::new();
            let mut ns_parts: Vec<NamespacePart> = Vec::new();
            for part in parts {
                match part {
                    DocumentPart::Include(part) => includes.push(part),
                    DocumentPart::NamespacePart(part) => ns_parts.push(part),
                }
            }
            Ok(Document {
                includes,
                ns: Namespace {
                    name: String::default(),
                    position: FilePosition { line: 1, column: 1 },
                    parts: ns_parts,
                },
            })
        }
        Ok((garbage, _)) => Err(ParseError::TrailingGarbage(garbage)),
        Err(error) => Err(ParseError::Nom(error)),
    }
}

fn parse_document_part(input: Span) -> IResult<Span, DocumentPart> {
    alt((
        map(parse_include, DocumentPart::Include),
        map(parse_namespace_part, DocumentPart::NamespacePart),
    ))(input)
}

pub fn parse_document_content(input: Span) -> IResult<Span, Vec<DocumentPart>> {
    preceded(
        ws,
        terminated(separated_list0(ws1, parse_document_part), ws),
    )(input)
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
        include common.ww;
        struct Person {
            name: String (length=1..50),
            age: Integer,
        }
        struct Group {
            name: String,
        }
        service Pinger {
            ping: None -> None,
            get_version: None -> String,
        }
    ";
    assert_eq!(
        parse_document(content),
        Ok(Document {
            includes: vec![Include {
                filename: "common.ww".to_string(),
                position: FilePosition {
                    line: 2,
                    column: 17
                },
            }],
            ns: Namespace {
                name: "".to_string(),
                position: FilePosition { line: 1, column: 1 },
                parts: vec![
                    NamespacePart::Struct(Struct {
                        name: "Person".to_string(),
                        position: FilePosition { line: 3, column: 9 },
                        generics: vec![],
                        fields: vec![
                            Field {
                                name: "name".to_string(),
                                position: FilePosition {
                                    line: 4,
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
                                    position: FilePosition {
                                        line: 4,
                                        column: 27,
                                    },
                                    name: "length".to_string(),
                                    value: Value::Range(Some(1), Some(50))
                                }],
                            },
                            Field {
                                name: "age".to_string(),
                                position: FilePosition {
                                    line: 5,
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
                        position: FilePosition { line: 7, column: 9 },
                        generics: vec![],
                        fields: vec![Field {
                            name: "name".to_string(),
                            position: FilePosition {
                                line: 8,
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
                        position: FilePosition {
                            line: 10,
                            column: 9
                        },
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
