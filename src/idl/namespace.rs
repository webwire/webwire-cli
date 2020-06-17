use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    multi::separated_list,
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::idl::common::{parse_identifier, ws, ws1, Span};
use crate::idl::fieldset::{parse_fieldset, Fieldset};
use crate::idl::r#enum::{parse_enum, Enum};
use crate::idl::r#struct::{parse_struct, Struct};
use crate::idl::service::{parse_service, Service};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Debug, PartialEq)]
pub enum NamespacePart {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
    Service(Service),
    Namespace(Namespace),
}

impl NamespacePart {
    pub fn name<'a>(&'a self) -> &'a str {
        let name = match self {
            Self::Enum(part) => &part.name,
            Self::Struct(part) => &part.name,
            Self::Fieldset(part) => &part.name,
            Self::Service(part) => &part.name,
            Self::Namespace(part) => &part.name,
        };
        name.as_str()
    }
}

#[derive(Debug, PartialEq)]
pub struct Namespace {
    pub name: String,
    pub parts: Vec<NamespacePart>,
}

fn parse_namespace_part(input: Span) -> IResult<Span, NamespacePart> {
    alt((
        map(parse_enum, NamespacePart::Enum),
        map(parse_fieldset, NamespacePart::Fieldset),
        map(parse_struct, NamespacePart::Struct),
        map(parse_service, NamespacePart::Service),
        map(parse_namespace, NamespacePart::Namespace),
    ))(input)
}

pub fn parse_namespace_content(input: Span) -> IResult<Span, Vec<NamespacePart>> {
    preceded(
        ws,
        terminated(separated_list(ws1, parse_namespace_part), ws),
    )(input)
}

pub fn parse_namespace(input: Span) -> IResult<Span, Namespace> {
    map(
        preceded(
            ws,
            preceded(
                terminated(tag("namespace"), ws1),
                cut(pair(
                    parse_identifier,
                    preceded(
                        preceded(ws, char('{')),
                        cut(terminated(parse_namespace_content, preceded(ws, char('}')))),
                    ),
                )),
            ),
        ),
        |(name, parts)| Namespace {
            name: name,
            parts: parts,
        },
    )(input)
}

#[test]
fn test_parse_namespace() {
    use crate::idl::field_option::FieldOption;
    use crate::idl::method::Method;
    use crate::idl::r#struct::Field;
    use crate::idl::r#type::Type;
    use crate::idl::value::Value;
    let content = "
        namespace test {
            struct Person {
                name: String (length=1..50),
                age: Integer
            }
            struct Group {
                name: String
            }
            service Pinger {
                ping(),
                get_version() -> String
            }
        }";
    assert_parse(
        parse_namespace(Span::new(content)),
        Namespace {
            name: "test".to_string(),
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
                                value: Value::Range(Some(1), Some(50)),
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
                    }],
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
            ],
        },
    )
}
