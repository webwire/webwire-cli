use nom::{
    IResult,
    bytes::complete::{tag},
    character::complete::char,
    combinator::{cut, map},
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
pub struct Service {
    pub name: String,
    // FIXME replace by in/out/err
    pub operations: Vec<String>,
}

fn parse_operations(input: &str) -> IResult<&str, Vec<String>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, preceded(ws, parse_identifier)),
            preceded(trailing_comma, preceded(ws, char('}')))
        ))
    )(input)
}

pub fn parse_service(input: &str) -> IResult<&str, Service> {
    map(
        preceded(
            terminated(tag("service"), ws1),
            cut(pair(
                parse_identifier,
                parse_operations,
            ))
        ),
        |(name, operations)| Service {
            name: name,
            operations: operations
        }
    )(input)
}

#[test]
fn test_parse_service_with_operations() {
    let contents = [
        // normal whitespaces
        "service Pinger { ping, get_version }",
        // no whitespace
        "service Pinger{ping,get_version}",
        // whitespace variants
        "service Pinger {ping,get_version}",
        "service Pinger{ping, get_version}",
        "service Pinger{ping ,get_version}",
        "service Pinger{ ping,get_version}",
        "service Pinger{ping,get_version }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_service(content),
            Ok(("", Service {
                name: "Pinger".to_string(),
                operations: vec![
                    "ping".to_string(),
                    "get_version".to_string(),
                ],
            }))
        )
    }
}
