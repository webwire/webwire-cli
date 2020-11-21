use std::str::FromStr;

#[cfg(test)]
use crate::idl::common::assert_parse;
use crate::idl::common::{parse_identifier, Span};
use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_a, is_not, tag},
    character::complete::{char, digit1, one_of},
    combinator::{cut, map, map_res, opt},
    error::context,
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Range(Option<i64>, Option<i64>),
    String(String),
    Identifier(String),
}

pub fn parse_boolean(input: Span) -> IResult<Span, bool> {
    alt((map(tag("false"), |_| false), map(tag("true"), |_| true)))(input)
}

pub fn parse_string(input: Span) -> IResult<Span, String> {
    context(
        "string",
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
                        )),
                    ),
                    String::from,
                ),
                char('\"'),
            )),
        ),
    )(input)
}

#[test]
fn test_parse_value_string() {
    assert_parse(
        parse_value(Span::new("\"hello\"")),
        Value::String("hello".to_string()),
    );
    assert_parse(
        parse_value(Span::new("\"hello world\"")),
        Value::String("hello world".to_string()),
    );
    assert_parse(
        parse_value(Span::new("\"hello\\nworld\"")),
        Value::String("hello\nworld".to_string()),
    );
    assert_parse(
        parse_value(Span::new("\"hello \\\"world\\\"\"")),
        Value::String("hello \"world\"".to_string()),
    );
    assert_parse(
        parse_value(Span::new("\"backspace\\\\\"")),
        Value::String("backspace\\".to_string()),
    );
}

pub fn parse_integer_dec(input: Span) -> IResult<Span, i64> {
    map_res(pair(opt(one_of("+-")), digit1), |(sign, number)| {
        i64::from_str_radix(format!("{}{}", sign.unwrap_or('+'), number).as_str(), 10)
    })(input)
}

pub fn parse_integer_hex(input: Span) -> IResult<Span, i64> {
    map_res(
        pair(
            opt(one_of("+-")),
            preceded(alt((tag("0x"), tag("0X"))), is_a("1234567890ABCDEFabcdef")),
        ),
        |(sign, number)| {
            i64::from_str_radix(format!("{}{}", sign.unwrap_or('+'), number).as_str(), 16)
        },
    )(input)
}

pub fn parse_integer(input: Span) -> IResult<Span, i64> {
    alt((parse_integer_hex, parse_integer_dec))(input)
}

pub fn parse_float(input: Span) -> IResult<Span, f64> {
    context(
        "float",
        map_res(
            pair(opt(one_of("+-")), separated_pair(digit1, char('.'), digit1)),
            |(sign, (a, b))| f64::from_str(format!("{}{}.{}", sign.unwrap_or('+'), a, b).as_str()),
        ),
    )(input)
}

pub fn parse_range(input: Span) -> IResult<Span, (Option<i64>, Option<i64>)> {
    context(
        "range",
        separated_pair(opt(parse_integer), tag(".."), opt(parse_integer)),
    )(input)
}

pub fn parse_value(input: Span) -> IResult<Span, Value> {
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
    assert_parse(parse_value(Span::new("true")), Value::Boolean(true));
    assert_parse(parse_value(Span::new("false")), Value::Boolean(false));
}

#[test]
fn test_parse_value_integer() {
    assert_parse(parse_value(Span::new("1337")), Value::Integer(1337));
    assert_parse(parse_value(Span::new("-42")), Value::Integer(-42));
    assert_parse(
        parse_value(Span::new("9223372036854775807")),
        Value::Integer(9223372036854775807),
    );
    assert_parse(
        parse_value(Span::new("-9223372036854775808")),
        Value::Integer(-9223372036854775808),
    );
    assert_parse(parse_value(Span::new("0xFF")), Value::Integer(0xFF));
    assert_parse(parse_value(Span::new("-0xFF")), Value::Integer(-0xFF));
}

#[test]
fn test_parse_value_integer_out_of_range() {
    use nom::error::ErrorKind;
    assert_eq!(
        parse_value(Span::new("9223372036854775808")),
        Err(nom::Err::Error(nom::error::Error {
            input: Span::new("9223372036854775808"),
            code: ErrorKind::TakeWhile1
        }))
    );
    assert_eq!(
        parse_value(Span::new("-9223372036854775809")),
        Err(nom::Err::Error(nom::error::Error {
            input: Span::new("-9223372036854775809"),
            code: ErrorKind::TakeWhile1
        }))
    );
}

#[test]
fn test_parse_value_float() {
    assert_parse(parse_value(Span::new("1337.0")), Value::Float(1337f64));
    assert_parse(parse_value(Span::new("13.37")), Value::Float(13.37f64));
    assert_parse(parse_value(Span::new("+13.37")), Value::Float(13.37f64));
    assert_parse(parse_value(Span::new("-13.37")), Value::Float(-13.37f64));
}

#[test]
fn test_parse_value_range() {
    assert_parse(
        parse_value(Span::new("0..1337")),
        Value::Range(Some(0), Some(1337)),
    );
    assert_parse(
        parse_value(Span::new("0..0xFF")),
        Value::Range(Some(0), Some(0xFF)),
    );
}
