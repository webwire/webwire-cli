use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::map,
    sequence::{preceded, tuple},
    IResult,
};

use crate::idl::common::{parse_identifier, ws, Span};
use crate::idl::r#type::{parse_opt_type, Type};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Debug, PartialEq)]
pub struct Method {
    pub name: String,
    pub input: Option<Type>,
    pub output: Option<Type>,
}

pub fn parse_method(input: Span) -> IResult<Span, Method> {
    map(
        tuple((
            parse_identifier,
            preceded(ws, preceded(char(':'), preceded(ws, parse_opt_type))),
            preceded(ws, preceded(tag("->"), preceded(ws, parse_opt_type))),
        )),
        |(name, input, output)| Method {
            name,
            input,
            output,
        },
    )(input)
}

#[test]
fn test_parse_method_0() {
    let contents = [
        // normal whitespace
        "ping: None -> None",
        // whitespace variants
        "ping:None->None",
        "ping :None->None",
        "ping: None->None",
        "ping:None ->None",
        "ping:None-> None",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "ping".to_string(),
                input: None,
                output: None,
            },
        )
    }
}

#[test]
fn test_parse_method_1() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        // normal whitespace
        "notify: Notification -> None",
        // whitespace variants
        "notify:Notification->None",
        "notify :Notification->None",
        "notify: Notification->None",
        "notify:Notification ->None",
        "notify:Notification-> None",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "notify".to_string(),
                input: Some(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Notification".to_string(),
                    generics: vec![],
                })),
                output: None,
            },
        )
    }
}

#[test]
fn test_parse_method_2() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        // normal whitespace
        "get_time: None -> Time",
        // whitespace variants
        "get_time:None->Time",
        "get_time :None->Time",
        "get_time: None->Time",
        "get_time:None ->Time",
        "get_time:None-> Time",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "get_time".to_string(),
                input: None,
                output: Some(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Time".to_string(),
                    generics: vec![],
                })),
            },
        )
    }
}

#[test]
fn test_parse_method_3() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        // normal whitespace
        "no_response: None -> Result<None, SomeError>",
        // whitespace variants
        "no_response:None->Result<None,SomeError>",
        "no_response :None->Result<None,SomeError>",
        "no_response: None->Result<None,SomeError>",
        "no_response:None ->Result<None,SomeError>",
        "no_response:None-> Result<None,SomeError>",
        "no_response:None->Result <None,SomeError>",
        "no_response:None->Result< None,SomeError>",
        "no_response:None->Result<None ,SomeError>",
        "no_response:None->Result<None, SomeError>",
        "no_response:None->Result<None,SomeError >",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "no_response".to_string(),
                input: None,
                output: Some(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Result".to_string(),
                    generics: vec![
                        Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "None".to_string(),
                            generics: vec![],
                        }),
                        Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "SomeError".to_string(),
                            generics: vec![],
                        }),
                    ],
                })),
            },
        )
    }
}

#[test]
fn test_parse_method_4() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        // normal whitespace
        "hello: HelloRequest -> Result<HelloResponse, HelloError>",
        // whitespace variants
        "hello:HelloRequest->Result<HelloResponse,HelloError>",
        "hello :HelloRequest->Result<HelloResponse,HelloError>",
        "hello: HelloRequest->Result<HelloResponse,HelloError>",
        "hello:HelloRequest ->Result<HelloResponse,HelloError>",
        "hello:HelloRequest-> Result<HelloResponse,HelloError>",
        "hello:HelloRequest->Result <HelloResponse,HelloError>",
        "hello:HelloRequest->Result< HelloResponse,HelloError>",
        "hello:HelloRequest->Result<HelloResponse ,HelloError>",
        "hello:HelloRequest->Result<HelloResponse, HelloError>",
        "hello:HelloRequest->Result<HelloResponse,HelloError >",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_method(Span::new(content)),
            Method {
                name: "hello".to_string(),
                input: Some(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "HelloRequest".to_string(),
                    generics: vec![],
                })),
                output: Some(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Result".to_string(),
                    generics: vec![
                        Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "HelloResponse".to_string(),
                            generics: vec![],
                        }),
                        Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "HelloError".to_string(),
                            generics: vec![],
                        }),
                    ],
                })),
            },
        )
    }
}
