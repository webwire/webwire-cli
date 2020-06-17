use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::char,
    combinator::{map, opt},
    sequence::{pair, preceded},
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

const WHITSPACE: &str = " \t\r\n";
const ALPHA_EXTRA: &str = "_";

pub fn ws(input: Span) -> IResult<Span, Span> {
    take_while(move |c| WHITSPACE.contains(c))(input)
}

pub fn ws1(input: Span) -> IResult<Span, Span> {
    take_while1(move |c| WHITSPACE.contains(c))(input)
}

pub fn trailing_comma(input: Span) -> IResult<Span, Option<char>> {
    opt(preceded(ws, char(',')))(input)
}

pub fn parse_identifier(input: Span) -> IResult<Span, String> {
    map(
        pair(
            take_while1(move |c: char| c.is_ascii_alphabetic()),
            take_while(move |c: char| c.is_ascii_alphanumeric() || ALPHA_EXTRA.contains(c)),
        ),
        |t| format!("{}{}", t.0, t.1),
    )(input)
}

#[cfg(test)]
pub(crate) fn assert_parse<'a, T: std::fmt::Debug + PartialEq>(
    output: IResult<LocatedSpan<&'a str>, T>,
    expected_value: T,
) {
    assert!(output.is_ok(), "{:?}", output);
    let output = output.unwrap();
    assert_eq!(output.0.fragment(), &"");
    assert_eq!(output.1, expected_value);
}

#[test]
fn test_parse_identifier() {
    assert_parse(parse_identifier(Span::new("test")), "test".to_string());
    assert_parse(
        parse_identifier(Span::new("test123")),
        "test123".to_string(),
    );
}

#[test]
fn test_parse_identifier_invalid() {
    use nom::error::ErrorKind;
    assert_eq!(
        parse_identifier(Span::new("123test")),
        Err(nom::Err::Error((
            Span::new("123test"),
            ErrorKind::TakeWhile1
        )))
    );
    assert_eq!(
        parse_identifier(Span::new("_test")),
        Err(nom::Err::Error((Span::new("_test"), ErrorKind::TakeWhile1)))
    );
}

pub fn parse_field_separator(input: Span) -> IResult<Span, char> {
    preceded(ws, char(','))(input)
}
