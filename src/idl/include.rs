use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::map,
    sequence::{preceded, terminated},
    IResult,
};

use crate::common::FilePosition;

use super::{
    common::{ws, ws1},
    Span,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Include {
    pub filename: String,
    pub position: FilePosition,
}

pub fn parse_include(input: Span) -> IResult<Span, Include> {
    preceded(
        ws,
        map(
            preceded(
                terminated(tag("include"), ws1),
                terminated(parse_filename, char(';')),
            ),
            |filename| Include {
                filename: filename.to_string(),
                position: filename.into(),
            },
        ),
    )(input)
}

pub fn parse_filename(input: Span) -> IResult<Span, Span> {
    take_while1(|c| c != ';')(input)
}

#[test]
fn test_parse_include() {
    use super::common::assert_parse;
    use super::*;
    let content = "include common.idl;";
    assert_parse(
        parse_include(Span::new(content)),
        Include {
            filename: String::from("common.idl"),
            position: FilePosition { line: 1, column: 9 },
        },
    );
}

#[test]
fn test_parse_include_with_directory() {
    use super::common::assert_parse;
    use super::*;
    let content = "include a/b/c.idl;";
    assert_parse(
        parse_include(Span::new(content)),
        Include {
            filename: String::from("a/b/c.idl"),
            position: FilePosition { line: 1, column: 9 },
        },
    );
}
