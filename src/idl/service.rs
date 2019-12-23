use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    multi::separated_list,
    sequence::{pair, preceded, terminated},
    IResult,
};

use crate::idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, ws1};

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

fn parse_endpoint(input: &str) -> IResult<&str, ServiceEndpoint> {
    map(
        pair(
            preceded(
                ws,
                terminated(alt((tag("inout"), tag("in"), tag("out"))), ws1),
            ),
            parse_identifier,
        ),
        |(inout, name)| ServiceEndpoint {
            in_: inout == "in" || inout == "inout",
            out: inout == "out" || inout == "inout",
            name: name,
        },
    )(input)
}

fn parse_endpoints(input: &str) -> IResult<&str, Vec<ServiceEndpoint>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, parse_endpoint),
            preceded(opt(preceded(ws, trailing_comma)), preceded(ws, char('}'))),
        )),
    )(input)
}

pub fn parse_service(input: &str) -> IResult<&str, Service> {
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
    assert_eq!(
        parse_endpoint("in f"),
        Ok((
            "",
            ServiceEndpoint {
                name: "f".to_string(),
                in_: true,
                out: false
            }
        ))
    );
    assert_eq!(
        parse_endpoint("out f"),
        Ok((
            "",
            ServiceEndpoint {
                name: "f".to_string(),
                in_: false,
                out: true
            }
        ))
    );
    assert_eq!(
        parse_endpoint("inout f"),
        Ok((
            "",
            ServiceEndpoint {
                name: "f".to_string(),
                in_: true,
                out: true
            }
        ))
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
        assert_eq!(
            parse_service(content),
            Ok((
                "",
                Service {
                    name: "Pinger".to_string(),
                    endpoints: vec![],
                }
            ))
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
        assert_eq!(
            parse_service(content),
            Ok((
                "",
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
            ))
        )
    }
}
