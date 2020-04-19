use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use crate::idl::common::{parse_identifier, ws, ws1};
use crate::idl::r#type::{parse_type, Type};

#[derive(Debug, PartialEq)]
pub struct Endpoint {
    pub name: String,
    pub request: Option<Type>,
    pub response: Option<Type>,
}

pub fn parse_endpoint(input: &str) -> IResult<&str, Endpoint> {
    map(
        tuple((
            preceded(tag("endpoint"), preceded(ws1, parse_identifier)),
            preceded(
                preceded(ws, char('(')),
                terminated(opt(preceded(ws, parse_type)), preceded(ws, char(')'))),
            ),
            opt(preceded(
                preceded(ws, tag("->")),
                preceded(ws, parse_type),
            )),
        )),
        |(name, request, response)| {
            Endpoint {
                name: name,
                request: request,
                response: response,
            }
        },
    )(input)
}

#[test]
fn test_parse_endpoint_0() {
    let contents = [
        // normal whitespace
        "endpoint ping()",
        // whitespace variants
        "endpoint ping ()",
        "endpoint ping( )",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok((
                "",
                Endpoint {
                    name: "ping".to_string(),
                    request: None,
                    response: None,
                }
            ))
        )
    }
}

#[test]
fn test_parse_endpoint_1() {
    let contents = [
        // normal whitespace
        "endpoint notify(Notification)",
        // whitespace variants
        "endpoint notify (Notification)",
        "endpoint notify( Notification)",
        "endpoint notify(Notification )",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok((
                "",
                Endpoint {
                    name: "notify".to_string(),
                    request: Some(Type::Named("Notification".to_string(), vec![])),
                    response: None,
                }
            ))
        )
    }
}

#[test]
fn test_parse_endpoint_2() {
    let contents = [
        // normal whitespace
        "endpoint get_time() -> Time",
        // whitespace variants
        "endpoint get_time()->Time",
        "endpoint get_time() ->Time",
        "endpoint get_time()-> Time",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok((
                "",
                Endpoint {
                    name: "get_time".to_string(),
                    request: None,
                    response: Some(Type::Named("Time".to_string(), vec![])),
                }
            ))
        )
    }
}

#[test]
fn test_parse_endpoint_3() {
    let contents = [
        // normal whitespace
        "endpoint no_response() -> Result<None, SomeError>",
        // whitespace variants
        "endpoint no_response() ->Result<None,SomeError>",
        "endpoint no_response()-> Result<None,SomeError>",
        "endpoint no_response()->Result <None,SomeError>",
        "endpoint no_response()->Result< None,SomeError>",
        "endpoint no_response()->Result<None ,SomeError>",
        "endpoint no_response()->Result<None, SomeError>",
        "endpoint no_response()->Result<None,SomeError >",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok((
                "",
                Endpoint {
                    name: "no_response".to_string(),
                    request: None,
                    response: Some(Type::Named("Result".to_string(), vec![
                        Type::Named("None".to_string(), vec![]),
                        Type::Named("SomeError".to_string(), vec![]),
                    ])),
                }
            ))
        )
    }
}

#[test]
fn test_parse_endpoint_4() {
    let contents = [
        // normal whitespace
        "endpoint hello(HelloRequest) -> Result<HelloResponse, HelloError>",
        // whitespace variants
        "endpoint hello(HelloRequest) ->Result<HelloResponse,HelloError>",
        "endpoint hello(HelloRequest)-> Result<HelloResponse,HelloError>",
        "endpoint hello(HelloRequest)->Result <HelloResponse,HelloError>",
        "endpoint hello(HelloRequest)->Result< HelloResponse,HelloError>",
        "endpoint hello(HelloRequest)->Result<HelloResponse ,HelloError>",
        "endpoint hello(HelloRequest)->Result<HelloResponse, HelloError>",
        "endpoint hello(HelloRequest)->Result<HelloResponse,HelloError >",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok((
                "",
                Endpoint {
                    name: "hello".to_string(),
                    request: Some(Type::Named("HelloRequest".to_string(), vec![])),
                    response: Some(Type::Named("Result".to_string(), vec![
                        Type::Named("HelloResponse".to_string(), vec![]),
                        Type::Named("HelloError".to_string(), vec![]),
                    ])),
                }
            ))
        )
    }
}
