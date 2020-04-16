use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    multi::separated_list,
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::idl::common::{parse_identifier, ws, ws1};
use crate::idl::endpoint::{parse_endpoint, Endpoint};
use crate::idl::fieldset::{parse_fieldset, Fieldset};
use crate::idl::operation::{parse_operation, Operation};
use crate::idl::r#enum::{parse_enum, Enum};
use crate::idl::r#struct::{parse_struct, Struct};
use crate::idl::service::{parse_service, Service};

#[derive(Debug, PartialEq)]
pub enum NamespacePart {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
    Operation(Operation),
    Endpoint(Endpoint),
    Service(Service),
    Namespace(Namespace),
}

#[derive(Debug, PartialEq)]
pub struct Namespace {
    pub name: String,
    pub parts: Vec<NamespacePart>,
}

fn parse_namespace_part(input: &str) -> IResult<&str, NamespacePart> {
    alt((
        map(parse_enum, NamespacePart::Enum),
        map(parse_fieldset, NamespacePart::Fieldset),
        map(parse_operation, NamespacePart::Operation),
        map(parse_struct, NamespacePart::Struct),
        map(parse_endpoint, NamespacePart::Endpoint),
        map(parse_service, NamespacePart::Service),
        map(parse_namespace, NamespacePart::Namespace),
    ))(input)
}

pub fn parse_namespace_content(input: &str) -> IResult<&str, Vec<NamespacePart>> {
    preceded(
        ws,
        terminated(separated_list(ws1, parse_namespace_part), ws),
    )(input)
}

pub fn parse_namespace<'a>(input: &'a str) -> IResult<&str, Namespace> {
    map(
        preceded(ws,
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
    use crate::idl::r#struct::Field;
    use crate::idl::r#type::Type;
    use crate::idl::service::ServiceEndpoint;
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
            endpoint ping()
            endpoint get_version() -> String
            service Pinger {
                in ping,
                inout get_version
            }
        }";
    assert_eq!(
        parse_namespace(content),
        Ok((
            "",
            Namespace {
                name: "test".to_string(),
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
            }
        ))
    )
}
