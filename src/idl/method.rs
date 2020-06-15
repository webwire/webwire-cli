use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{map, opt},
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::idl::common::{parse_identifier, ws, ws1, Span};
use crate::idl::r#type::{parse_type, Type};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Debug, PartialEq)]
pub struct Method {
    pub name: String,
    pub request: Option<Type>,
    pub response: Option<Type>,
}

pub fn parse_method(input: Span) -> IResult<Span, Method> {
    map(
        tuple((
            parse_identifier,
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
            Method {
                name: name,
                request: request,
                response: response,
            }
        },
    )(input)
}

#[test]
fn test_parse_method_0() {
    let contents = [
        // normal whitespace
        "ping()",
        // whitespace variants
        "ping ()",
        "ping( )",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "ping".to_string(),
                request: None,
                response: None,
            }
        )
    }
}

#[test]
fn test_parse_method_1() {
    let contents = [
        // normal whitespace
        "notify(Notification)",
        // whitespace variants
        "notify (Notification)",
        "notify( Notification)",
        "notify(Notification )",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "notify".to_string(),
                request: Some(Type::Named("Notification".to_string(), vec![])),
                response: None,
            }
        )
    }
}

#[test]
fn test_parse_method_2() {
    let contents = [
        // normal whitespace
        "get_time() -> Time",
        // whitespace variants
        "get_time()->Time",
        "get_time() ->Time",
        "get_time()-> Time",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "get_time".to_string(),
                request: None,
                response: Some(Type::Named("Time".to_string(), vec![])),
            }
        )
    }
}

#[test]
fn test_parse_method_3() {
    let contents = [
        // normal whitespace
        "no_response() -> Result<None, SomeError>",
        // whitespace variants
        "no_response() ->Result<None,SomeError>",
        "no_response()-> Result<None,SomeError>",
        "no_response()->Result <None,SomeError>",
        "no_response()->Result< None,SomeError>",
        "no_response()->Result<None ,SomeError>",
        "no_response()->Result<None, SomeError>",
        "no_response()->Result<None,SomeError >",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "no_response".to_string(),
                request: None,
                response: Some(Type::Named("Result".to_string(), vec![
                    Type::Named("None".to_string(), vec![]),
                    Type::Named("SomeError".to_string(), vec![]),
                ])),
            }
        )
    }
}

#[test]
fn test_parse_method_4() {
    let contents = [
        // normal whitespace
        "hello(HelloRequest) -> Result<HelloResponse, HelloError>",
        // whitespace variants
        "hello(HelloRequest) ->Result<HelloResponse,HelloError>",
        "hello(HelloRequest)-> Result<HelloResponse,HelloError>",
        "hello(HelloRequest)->Result <HelloResponse,HelloError>",
        "hello(HelloRequest)->Result< HelloResponse,HelloError>",
        "hello(HelloRequest)->Result<HelloResponse ,HelloError>",
        "hello(HelloRequest)->Result<HelloResponse, HelloError>",
        "hello(HelloRequest)->Result<HelloResponse,HelloError >",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "hello".to_string(),
                request: Some(Type::Named("HelloRequest".to_string(), vec![])),
                response: Some(Type::Named("Result".to_string(), vec![
                    Type::Named("HelloResponse".to_string(), vec![]),
                    Type::Named("HelloError".to_string(), vec![]),
                ])),
            }
        )
    }
}
