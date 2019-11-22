use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list,
    sequence::{pair, preceded, terminated, tuple},
};

use crate::idl::common::{
    ws,
    ws1,
    parse_identifier,
    parse_field_separator,
    trailing_comma,
};

#[derive(Debug, PartialEq)]
pub struct Endpoint {
    pub name: String,
    pub request: Option<String>,
    pub response: Option<String>,
    pub error: Option<String>
}

pub fn parse_endpoint(input: &str) -> IResult<&str, Endpoint> {
    map(
        tuple((
            preceded(
                tag("endpoint"),
                preceded(ws1, parse_identifier)
            ),
            preceded(
                preceded(ws, char('(')),
                terminated(
                    opt(preceded(ws, parse_identifier)),
                    preceded(ws, char(')'))
                )
            ),
            opt(
                preceded(
                    preceded(ws, tag("->")),
                    pair(
                        preceded(ws, parse_identifier),
                        opt(
                            preceded(ws, preceded(char('|'), preceded(ws,
                                parse_identifier
                            )))
                        )
                    )
                )
            )
        )),
        |(name, request, response)| {
            if let Some(response) = response {
                Endpoint {
                    name: name,
                    request: request,
                    response: if response.0 != "None" { Some(response.0) } else { None },
                    error: response.1
                }
            } else {
                Endpoint {
                    name: name,
                    request: request,
                    response: None,
                    error: None
                }
            }
        }
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
            Ok(("", Endpoint {
                name: "ping".to_string(),
                request: None,
                response: None,
                error: None
            }))
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
            Ok(("", Endpoint {
                name: "notify".to_string(),
                request: Some("Notification".to_string()),
                response: None,
                error: None
            }))
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
            Ok(("", Endpoint {
                name: "get_time".to_string(),
                request: None,
                response: Some("Time".to_string()),
                error: None
            }))
        )
    }
}

#[test]
fn test_parse_endpoint_3() {
    let contents = [
        // normal whitespace
        "endpoint no_response() -> None | SomeError",
        // whitespace variants
        "endpoint no_response() ->None|SomeError",
        "endpoint no_response()-> None|SomeError",
        "endpoint no_response()->None |SomeError",
        "endpoint no_response()->None| SomeError",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok(("", Endpoint {
                name: "no_response".to_string(),
                request: None,
                response: None,
                error: Some("SomeError".to_string())
            }))
        )
    }
}

#[test]
fn test_parse_endpoint_4() {
    let contents = [
        // normal whitespace
        "endpoint hello(HelloRequest) -> HelloResponse | HelloError",
        // whitespace variants
        "endpoint hello(HelloRequest) ->HelloResponse|HelloError",
        "endpoint hello(HelloRequest)-> HelloResponse|HelloError",
        "endpoint hello(HelloRequest)->HelloResponse |HelloError",
        "endpoint hello(HelloRequest)->HelloResponse| HelloError",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_endpoint(content),
            Ok(("", Endpoint {
                name: "hello".to_string(),
                request: Some("HelloRequest".to_string()),
                response: Some("HelloResponse".to_string()),
                error: Some("HelloError".to_string())
            }))
        )
    }
}
