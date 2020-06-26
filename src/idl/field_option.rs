use nom::{
    character::complete::char,
    combinator::{cut, map},
    error::context,
    multi::separated_list,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

#[cfg(test)]
use crate::idl::common::assert_parse;
use crate::idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, Span};
use crate::idl::r#value::{parse_value, Value};

#[derive(Debug, PartialEq)]
pub struct FieldOption {
    pub name: String,
    pub value: Value,
}

pub fn parse_field_options(input: Span) -> IResult<Span, Vec<FieldOption>> {
    context(
        "options",
        preceded(
            preceded(ws, char('(')),
            cut(terminated(
                separated_list(parse_field_separator, parse_field_option),
                preceded(trailing_comma, preceded(ws, char(')'))),
            )),
        ),
    )(input)
}

fn parse_field_option(input: Span) -> IResult<Span, FieldOption> {
    map(
        separated_pair(
            preceded(ws, parse_identifier),
            preceded(ws, char('=')),
            preceded(ws, parse_value),
        ),
        |(name, value)| FieldOption { name, value },
    )(input)
}

#[test]
fn test_parse_field_options_0() {
    let contents = ["()", "( )", "(,)", "( ,)", "(, )"];
    for content in contents.iter() {
        assert_parse(parse_field_options(Span::new(content)), vec![]);
    }
}

#[test]
fn test_parse_field_options_1() {
    let contents = [
        "(foo=42)",
        "(foo= 42)",
        "(foo=42 )",
        "( foo=42)",
        "(foo=42,)",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_field_options(Span::new(content)),
            vec![FieldOption {
                name: "foo".to_owned(),
                value: Value::Integer(42),
            }],
        );
    }
}

#[test]
fn test_parse_field_options_2() {
    let contents = [
        "(foo=42,bar=\"epic\")",
        "(foo= 42, bar= \"epic\")",
        "( foo=42,bar=\"epic\" )",
        "( foo= 42, bar= \"epic\" )",
        "( foo= 42, bar= \"epic\", )",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_field_options(Span::new(content)),
            vec![
                FieldOption {
                    name: "foo".to_owned(),
                    value: Value::Integer(42),
                },
                FieldOption {
                    name: "bar".to_owned(),
                    value: Value::String("epic".to_string()),
                },
            ],
        );
    }
}
