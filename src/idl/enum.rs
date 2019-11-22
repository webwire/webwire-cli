use nom::{
    IResult,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map},
    error::context,
    multi::separated_list,
    sequence::{pair, preceded, terminated}
};

use crate::idl::common::{
    ws,
    ws1,
    parse_identifier,
    parse_field_separator,
    trailing_comma,
};

#[derive(Debug, PartialEq)]
pub struct Enum {
    pub name: String,
    pub values: Vec<String>
}

pub fn parse_enum(input: &str) -> IResult<&str, Enum> {
    map(
        pair(
            preceded(
                terminated(tag("enum"), ws1),
                parse_identifier,
            ),
            parse_enum_values
        ),
        |t| Enum {
            name: t.0.to_string(),
            values: t.1
        }
    )(input)
}

fn parse_enum_values(input: &str) -> IResult<&str, Vec<String>> {
    context(
        "enum_values",
        preceded(
            preceded(ws, char('{')),
            cut(terminated(
                separated_list(parse_field_separator, preceded(ws, parse_identifier)),
                preceded(trailing_comma, preceded(ws, char('}')))
            ))
        )
    )(input)
}

#[test]
fn test_parse_enum_0() {
    let contents = [
        // minimal whitespace
        "enum Nothing{}",
        // normal whitespace
        "enum Nothing {}",
        // whitespace variants
        "enum Nothing { }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_enum(content),
            Ok(("", Enum {
                name: "Nothing".to_string(),
                values: vec![],
            }))
        )
    }
}

#[test]
fn test_parse_enum_1() {
    let contents = [
        // minimal whitespace
        "enum OneThing{Thing}",
        // whitespace variants
        "enum OneThing {Thing}",
        "enum OneThing{ Thing}",
        "enum OneThing{Thing }",
        "enum OneThing { Thing }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_enum(content),
            Ok(("", Enum {
                name: "OneThing".to_string(),
                values: vec!["Thing".to_string()],
            }))
        )
    }
}

#[test]
fn test_parse_enum_2() {
    let contents = [
        // minimal whitespace
        "enum Direction{Left,Right}",
        // normal whitespace
        "enum Direction { Left, Right }",
        // whitespace variants
        "enum Direction {Left,Right}",
        "enum Direction{ Left,Right}",
        "enum Direction{Left ,Right}",
        "enum Direction{Left, Right}",
        "enum Direction{Left,Right }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_enum(content),
            Ok(("", Enum {
                name: "Direction".to_string(),
                values: vec!["Left".to_string(), "Right".to_string()],
            }))
        )
    }
}
