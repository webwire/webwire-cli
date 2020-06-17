use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    error::context,
    multi::separated_list,
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, ws1, Span};
use crate::idl::method::{parse_method, Method};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Debug, PartialEq)]
pub struct Service {
    pub name: String,
    pub methods: Vec<Method>,
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
            |(name, methods)| Service { name, methods },
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
            },
        )
    }
}

#[test]
fn test_parse_service() {
    use crate::idl::r#type::Type;
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
            },
        )
    }
}
