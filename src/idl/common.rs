use nom::{
    IResult,
    bytes::complete::{take_while, take_while1},
    character::complete::char,
    combinator::{map, opt},
    sequence::{pair, preceded}
};

const WHITSPACE: &str = " \t\r\n";
const ALPHA_EXTRA: &str = "_";

pub fn ws(input: &str) -> IResult<&str, &str> {
    take_while(move |c| WHITSPACE.contains(c))(input)
}

pub fn ws1(input: &str) -> IResult<&str, &str> {
    take_while1(move |c| WHITSPACE.contains(c))(input)
}

pub fn trailing_comma(input: &str) -> IResult<&str, Option<char>> {
    opt(preceded(
        ws,
        char(',')
    ))(input)
}

pub fn parse_identifier(input: &str) -> IResult<&str, String> {
    preceded(
        ws,
        map(
            pair(
                take_while1(move |c: char| c.is_ascii_alphabetic()),
                take_while(move |c: char| c.is_ascii_alphanumeric() || ALPHA_EXTRA.contains(c)),
            ),
            |t| format!("{}{}", t.0, t.1)
        )
    )(input)
}

#[test]
fn test_parse_identifier() {
    assert_eq!(
        parse_identifier("test"),
        Ok(("", "test".to_string()))
    );
    assert_eq!(
        parse_identifier("test123"),
        Ok(("", "test123".to_string()))
    );
}

#[test]
fn test_parse_identifier_invalid() {
    use nom::error::ErrorKind;
    assert_eq!(
        parse_identifier("123test"),
        Err(nom::Err::Error(("123test", ErrorKind::TakeWhile1)))
    );
    assert_eq!(
        parse_identifier("_test"),
        Err(nom::Err::Error(("_test", ErrorKind::TakeWhile1)))
    );
}

pub fn parse_field_separator(input: &str) -> IResult<&str, char> {
    preceded(ws, char(','))(input)
}
