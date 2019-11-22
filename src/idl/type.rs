use nom::{
    IResult,
    branch::alt,
    character::complete::char,
    combinator::{cut, map},
    error::context,
    sequence::{preceded, separated_pair, terminated}
};

use crate::idl::common::{
    parse_identifier,
    ws,
};

#[derive(Debug, PartialEq)]
pub enum Type {
    Named(String),
    Array(String),
    Map(String, String),
}

fn parse_type_array(input: &str) -> IResult<&str, Type> {
    context("array",
        preceded(
            char('['),
            cut(terminated(
                preceded(ws, map(parse_identifier, Type::Array)),
                preceded(ws, char(']'))
            ))
        )
    )(input)
}

fn parse_type_map_inner(input: &str) -> IResult<&str, Type> {
    map(
        separated_pair(
            preceded(ws, parse_identifier),
            cut(preceded(ws, char(':'))),
            preceded(ws, parse_identifier),
        ),
        |types| Type::Map(types.0.to_string(), types.1.to_string())
    )(input)
}

fn parse_type_map(input: &str) -> IResult<&str, Type> {
    context("map",
        preceded(
            char('{'),
            cut(terminated(
                preceded(ws, parse_type_map_inner),
                preceded(ws, char('}'))
            ))
        )
    )(input)
}

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    preceded(ws,
        alt((
            map(parse_identifier, Type::Named),
            parse_type_array,
            parse_type_map,
        ))
    )(input)
}

#[test]
fn test_parse_type_array() {
    let contents = [
        "[UUID]",
        "[ UUID]",
        "[UUID ]",
        "[ UUID ]",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Array("UUID".to_string())))
        );
    }
}

#[test]
fn test_parse_type_map() {
    let contents = [
        "{UUID:String}",
        "{ UUID:String}",
        "{UUID:String }",
        "{UUID :String}",
        "{UUID: String}",
        "{ UUID : String }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Map(
                "UUID".to_string(),
                "String".to_string()
            )))
        );
    }
}

