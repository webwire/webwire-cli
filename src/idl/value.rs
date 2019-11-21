use std::str::FromStr;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{escaped_transform, is_not, is_a, tag},
    character::complete::{char, digit1, one_of},
    combinator::{cut, map, map_res, opt},
    error::context,
    sequence::{pair, preceded, separated_pair, terminated}
};

use crate::idl::common::parse_identifier;

#[derive(Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Range(Option<i64>, Option<i64>),
    String(String),
    Identifier(String),
}

pub fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag("false"), |_| false),
        map(tag("true"), |_| true)
    ))(input)
}

pub fn parse_string(input: &str) -> IResult<&str, String> {
    context("string",
        preceded(
            char('\"'),
            cut(terminated(
                map(
                    escaped_transform(
                        is_not("\\\"\n"),
                        '\\',
                        alt((
                            map(tag("\\"), |_| "\\"),
                            map(tag("\""), |_| "\""),
                            map(tag("n"), |_| "\n"),
                        ))
                    ),
                    String::from
                ),
                char('\"')
            ))
        )
    )(input)
}

#[test]
fn test_parse_value_string() {
    assert_eq!(parse_value("\"hello\""), Ok(("", Value::String("hello".to_string()))));
    assert_eq!(parse_value("\"hello world\""), Ok(("", Value::String("hello world".to_string()))));
    assert_eq!(parse_value("\"hello\\nworld\""), Ok(("", Value::String("hello\nworld".to_string()))));
    assert_eq!(parse_value("\"hello \\\"world\\\"\""), Ok(("", Value::String("hello \"world\"".to_string()))));
    assert_eq!(parse_value("\"backspace\\\\\""), Ok(("", Value::String("backspace\\".to_string()))));
}

pub fn parse_integer_dec(input: &str) -> IResult<&str, i64> {
    map_res(
        pair(
            opt(one_of("+-")),
            digit1,
        ),
        |(sign, number)| {
            i64::from_str_radix(format!("{}{}", sign.unwrap_or('+'), number).as_str(), 10)
        }
    )(input)
}

pub fn parse_integer_hex(input: &str) -> IResult<&str, i64> {
    map_res(
        pair(
            opt(one_of("+-")),
            preceded(
                alt((tag("0x"), tag("0X"))),
                is_a("1234567890ABCDEFabcdef"),
            )
        ),
        |(sign, number)| {
            i64::from_str_radix(format!("{}{}", sign.unwrap_or('+'), number).as_str(), 16)
        }
    )(input)
}

pub fn parse_integer(input: &str) -> IResult<&str, i64> {
    alt((
        parse_integer_hex,
        parse_integer_dec,
    ))(input)
}

pub fn parse_float(input: &str) -> IResult<&str, f64> {
    context("float",
        map_res(
            separated_pair(
                digit1,
                char('.'),
                digit1
            ),
            |(a, b)| {
                f64::from_str(format!("{}.{}", a, b).as_str())
            }
        ),
    )(input)
}

pub fn parse_range(input: &str) -> IResult<&str, (Option<i64>, Option<i64>)> {
    context("range",
        separated_pair(
            opt(parse_integer),
            tag(".."),
            opt(parse_integer)
        )
    )(input)
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    alt((
        map(parse_boolean, Value::Boolean),
        map(parse_range, |(min, max)| Value::Range(min, max)),
        map(parse_float, Value::Float),
        map(parse_integer, Value::Integer),
        map(parse_string, Value::String),
        map(parse_identifier, Value::Identifier),
    ))(input)
}

#[test]
fn test_parse_value_boolean() {
    assert_eq!(parse_value("true"), Ok(("", Value::Boolean(true))));
    assert_eq!(parse_value("false"), Ok(("", Value::Boolean(false))));
}

#[test]
fn test_parse_value_integer() {
    assert_eq!(parse_value("1337"), Ok(("", Value::Integer(1337))));
    assert_eq!(parse_value("-42"), Ok(("", Value::Integer(-42))));
    assert_eq!(parse_value("9223372036854775807"), Ok(("", Value::Integer(9223372036854775807))));
    assert_eq!(parse_value("-9223372036854775808"), Ok(("", Value::Integer(-9223372036854775808))));
    assert_eq!(parse_value("0xFF"), Ok(("", Value::Integer(0xFF))));
    assert_eq!(parse_value("-0xFF"), Ok(("", Value::Integer(-0xFF))));
}

#[test]
fn test_parse_value_integer_out_of_range() {
    use nom::error::ErrorKind;
    assert_eq!(
        parse_value("9223372036854775808"),
        Err(nom::Err::Error(("9223372036854775808", ErrorKind::TakeWhile1)))
    );
    assert_eq!(
        parse_value("-9223372036854775809"),
        Err(nom::Err::Error(("-9223372036854775809", ErrorKind::TakeWhile1)))
    );
}

#[test]
fn test_parse_value_float() {
    assert_eq!(parse_value("1337.0"), Ok(("", Value::Float(1337f64))));
    assert_eq!(parse_value("13.37"), Ok(("", Value::Float(13.37f64))));
}

/*
#[test]
fn test_parse_value_string() {
    assert_eq!(parse_value("\"hello\""), Ok(("", Value::String("hello".to_string()))));
    assert_eq!(parse_value("\"hello world\""), Ok(("", Value::String("hello world".to_string()))));
}
*/

#[test]
fn test_parse_value_range() {
    assert_eq!(parse_value("0..1337"), Ok(("", Value::Range(Some(0), Some(1337)))));
    // TODO add support for hexadecimal numbers
    //assert_eq!(parse_value("0..0xFF"), Ok(("", Value::Range(0, 0xFF))));
}

