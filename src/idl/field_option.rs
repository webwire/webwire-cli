use nom::{
    character::complete::char,
    combinator::{cut, map},
    error::context,
    multi::separated_list0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

#[cfg(test)]
use crate::idl::common::assert_parse;
use crate::idl::r#value::{parse_value, Value};
use crate::{
    common::FilePosition,
    idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, Span},
};

#[derive(Debug, PartialEq)]
pub struct FieldOption {
    pub position: FilePosition,
    pub name: String,
    pub value: Value,
}

pub fn parse_field_options(input: Span) -> IResult<Span, Vec<FieldOption>> {
    context(
        "options",
        preceded(
            preceded(ws, char('(')),
            cut(terminated(
                separated_list0(parse_field_separator, preceded(ws, parse_field_option)),
                preceded(trailing_comma, preceded(ws, char(')'))),
            )),
        ),
    )(input)
}

fn parse_field_option(input: Span) -> IResult<Span, FieldOption> {
    map(
        separated_pair(
            parse_identifier,
            preceded(ws, char('=')),
            preceded(ws, parse_value),
        ),
        |(name, value)| FieldOption {
            position: input.into(),
            name,
            value,
        },
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
        ("(foo=42)", 2),
        ("(foo= 42)", 2),
        ("(foo=42 )", 2),
        ("( foo=42)", 3),
        ("(foo=42,)", 2),
    ];
    for (content, foo_column) in contents.iter() {
        assert_parse(
            parse_field_options(Span::new(content)),
            vec![FieldOption {
                position: FilePosition {
                    line: 1,
                    column: *foo_column,
                },
                name: "foo".to_owned(),
                value: Value::Integer(42),
            }],
        );
    }
}

#[test]
fn test_parse_field_options_2() {
    let contents = [
        ("(foo=42,bar=\"epic\")", (2, 9)),
        ("(foo= 42, bar= \"epic\")", (2, 11)),
        ("( foo=42,bar=\"epic\" )", (3, 10)),
        ("( foo= 42, bar= \"epic\" )", (3, 12)),
        ("( foo= 42, bar= \"epic\", )", (3, 12)),
    ];
    for (content, (foo_column, bar_column)) in contents.iter() {
        assert_parse(
            parse_field_options(Span::new(content)),
            vec![
                FieldOption {
                    position: FilePosition {
                        line: 1,
                        column: *foo_column,
                    },
                    name: "foo".to_owned(),
                    value: Value::Integer(42),
                },
                FieldOption {
                    position: FilePosition {
                        line: 1,
                        column: *bar_column,
                    },
                    name: "bar".to_owned(),
                    value: Value::String("epic".to_string()),
                },
            ],
        );
    }
}
