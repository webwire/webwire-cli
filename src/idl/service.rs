use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    error::context,
    multi::separated_list,
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::common::FilePosition;
use crate::idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, ws1, Span};
use crate::idl::method::{parse_method, Method};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Debug, PartialEq)]
pub struct Service {
    pub name: String,
    pub methods: Vec<Method>,
    pub position: FilePosition,
}

fn parse_methods(input: Span) -> IResult<Span, Vec<Method>> {
    context(
        "methods",
        preceded(
            preceded(ws, char('{')),
            cut(terminated(
                separated_list(parse_field_separator, preceded(ws, parse_method)),
                preceded(trailing_comma, preceded(ws, char('}'))),
            )),
        ),
    )(input)
}

pub fn parse_service(input: Span) -> IResult<Span, Service> {
    context(
        "service",
        map(
            preceded(
                terminated(tag("service"), ws1),
                cut(pair(parse_identifier, parse_methods)),
            ),
            |(name, methods)| Service {
                name,
                methods,
                position: input.into(),
            },
        ),
    )(input)
}

#[test]
fn test_parse_service_no_endpoints() {
    let contents = [
        // normal whitespaces
        "service Pinger {}",
        // whitespace variants
        "service Pinger{}",
        "service Pinger{ }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_service(Span::new(content)),
            Service {
                name: "Pinger".to_string(),
                methods: vec![],
                position: FilePosition { line: 1, column: 1 },
            },
        )
    }
}

#[test]
fn test_parse_service() {
    use crate::idl::r#type::{ Type, TypeRef };
    let contents = [
        // normal whitespaces
        "service Pinger { ping(), get_version() -> String }",
        // whitespace variants
        "service Pinger{ping(),get_version()->String}",
        "service Pinger {ping(),get_version()->String}",
        "service Pinger{ping (),get_version()->String}",
        "service Pinger{ping( ),get_version()->String}",
        "service Pinger{ping() ,get_version()->String}",
        "service Pinger{ping(), get_version()->String}",
        "service Pinger{ping(),get_version ()->String}",
        "service Pinger{ping(),get_version( )->String}",
        "service Pinger{ping(),get_version() ->String}",
        "service Pinger{ping(),get_version()-> String}",
        "service Pinger{ping(),get_version()->String }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_service(Span::new(content)),
            Service {
                name: "Pinger".to_string(),
                position: FilePosition { line: 1, column: 1 },
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
                            generics: vec![],
                        })),
                    },
                ],
            },
        )
    }
}
