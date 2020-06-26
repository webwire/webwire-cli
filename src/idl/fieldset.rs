use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    multi::separated_list,
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::common::FilePosition;
use crate::idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, ws1, Span};
use crate::idl::r#type::{parse_type_ref, TypeRef};

#[cfg(test)]
use crate::idl::common::assert_parse;

#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub optional: bool,
}

#[derive(Debug, PartialEq)]
pub struct Fieldset {
    pub name: String,
    pub r#struct: TypeRef,
    pub fields: Vec<Field>,
    pub position: FilePosition,
}

fn parse_field(input: Span) -> IResult<Span, Field> {
    map(
        pair(preceded(ws, parse_identifier), preceded(ws, opt(char('?')))),
        |(name, optional)| Field {
            name,
            optional: optional != None,
        },
    )(input)
}

fn parse_fields(input: Span) -> IResult<Span, Vec<Field>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, parse_field),
            preceded(trailing_comma, preceded(ws, char('}'))),
        )),
    )(input)
}

pub fn parse_fieldset(input: Span) -> IResult<Span, Fieldset> {
    map(
        preceded(
            terminated(tag("fieldset"), ws1),
            cut(pair(
                separated_pair(
                    preceded(ws, parse_identifier),
                    preceded(ws, tag("for")),
                    preceded(ws1, parse_type_ref),
                ),
                parse_fields,
            )),
        ),
        |((name, struct_), fields)| Fieldset {
            name,
            r#struct: struct_,
            fields,
            position: input.into(),
        },
    )(input)
}

#[test]
fn test_parse_fieldset_0() {
    let contents = [
        // minimal whitespace
        "fieldset PersonName for Person{}",
        // normal whitespace
        "fieldset PersonName for Person {}",
        // whitespace variants
        "fieldset PersonName for Person { }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_fieldset(Span::new(content)),
            Fieldset {
                name: "PersonName".to_string(),
                position: FilePosition { line: 1, column: 1 },
                r#struct: TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Person".to_string(),
                    generics: vec![],
                },
                fields: vec![],
            },
        )
    }
}

#[test]
fn test_parse_fieldset_1() {
    let contents = [
        // minimal whitespace
        "fieldset PersonName for Person{name}",
        // whitespace variants
        "fieldset PersonName for Person {name}",
        "fieldset PersonName for Person{ name}",
        "fieldset PersonName for Person{name }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_fieldset(Span::new(content)),
            Fieldset {
                name: "PersonName".to_string(),
                position: FilePosition { line: 1, column: 1 },
                r#struct: TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Person".to_string(),
                    generics: vec![],
                },
                fields: vec![Field {
                    name: "name".to_string(),
                    optional: false,
                }],
            },
        )
    }
}

#[test]
fn test_parse_fieldset_2() {
    let contents = [
        // minimal whitespace
        "fieldset PersonName for Person{name,age?}",
        // normal whitespace
        "fieldset PersonName for Person { name, age? }",
        // whitespace variants
        "fieldset PersonName for Person {name,age?}",
        "fieldset PersonName for Person{ name,age?}",
        "fieldset PersonName for Person{name ,age?}",
        "fieldset PersonName for Person{name, age?}",
        "fieldset PersonName for Person{name,age ?}",
        "fieldset PersonName for Person{name,age? }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_fieldset(Span::new(content)),
            Fieldset {
                name: "PersonName".to_string(),
                position: FilePosition { line: 1, column: 1 },
                r#struct: TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Person".to_string(),
                    generics: vec![],
                },
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        optional: false,
                    },
                    Field {
                        name: "age".to_string(),
                        optional: true,
                    },
                ],
            },
        )
    }
}
