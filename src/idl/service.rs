use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    multi::separated_list,
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::idl::common::{
    parse_field_separator,
    parse_identifier,
    trailing_comma,
    ws,
    ws1,
    Span
};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Debug, PartialEq)]
pub struct Service {
    pub name: String,
    pub endpoints: Vec<ServiceEndpoint>,
}

#[derive(Debug, PartialEq)]
pub struct ServiceEndpoint {
    pub in_: bool,
    pub out: bool,
    pub name: String,
}

fn parse_endpoint(input: Span) -> IResult<Span, ServiceEndpoint> {
    map(
        pair(
            preceded(
                ws,
                terminated(alt((tag("inout"), tag("in"), tag("out"))), ws1),
            ),
            parse_identifier,
        ),
        |(inout, name)| ServiceEndpoint {
            in_: inout.fragment() == &"in" || inout.fragment() == &"inout",
            out: inout.fragment() == &"out" || inout.fragment() == &"inout",
            name: name,
        },
    )(input)
}

fn parse_endpoints(input: Span) -> IResult<Span, Vec<ServiceEndpoint>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, parse_endpoint),
            preceded(opt(preceded(ws, trailing_comma)), preceded(ws, char('}'))),
        )),
    )(input)
}

pub fn parse_service(input: Span) -> IResult<Span, Service> {
    map(
        preceded(
            terminated(tag("service"), ws1),
            cut(pair(parse_identifier, parse_endpoints)),
        ),
        |(name, endpoints)| Service {
            name: name,
            endpoints: endpoints,
        },
    )(input)
}

#[test]
fn test_parse_endpoint() {
    assert_parse(
        parse_endpoint(Span::new("in f")),
        ServiceEndpoint {
            name: "f".to_string(),
            in_: true,
            out: false
        }
    );
    assert_parse(
        parse_endpoint(Span::new("out f")),
        ServiceEndpoint {
            name: "f".to_string(),
            in_: false,
            out: true
        }
    );
    assert_parse(
        parse_endpoint(Span::new("inout f")),
        ServiceEndpoint {
            name: "f".to_string(),
            in_: true,
            out: true
        }
    );
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
                endpoints: vec![],
            }
        )
    }
}

#[test]
fn test_parse_service() {
    let contents = [
        // normal whitespaces
        "service Pinger { in ping, inout get_version }",
        // whitespace variants
        "service Pinger{ in ping,inout get_version}",
        "service Pinger{in ping ,inout get_version}",
        "service Pinger{in ping, inout get_version}",
        "service Pinger{in ping,inout get_version }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_service(Span::new(content)),
            Service {
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
            }
        )
    }
}
