use nom::{
    IResult,
    bytes::complete::{tag},
    character::complete::char,
    combinator::{cut, map, opt},
    multi::separated_list,
    sequence::{pair, preceded, separated_pair, terminated}
};

use crate::idl::common::{
    parse_identifier,
    parse_field_separator,
    trailing_comma,
    ws,
    ws1,
};

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub optional: bool
}

#[derive(Debug, PartialEq)]
pub struct Fieldset {
    pub name: String,
    pub struct_name: String,
    pub fields: Vec<Field>,
}

fn parse_field(input: &str) -> IResult<&str, Field> {
    map(
        pair(
            parse_identifier,
            opt(char('?'))
        ),
        |(name, optional)| Field {
            name: name.to_string(),
            optional: optional != None
        }
    )(input)
}

fn parse_fields(input: &str) -> IResult<&str, Vec<Field>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, parse_field),
            preceded(trailing_comma, preceded(ws, char('}')))
        ))
    )(input)
}

pub fn parse_fieldset(input: &str) -> IResult<&str, Fieldset> {
    map(
        preceded(
            terminated(tag("fieldset"), ws1),
            cut(pair(
                separated_pair(
                    parse_identifier,
                    preceded(ws, tag("for")),
                    preceded(ws1, parse_identifier),
                ),
                parse_fields
            ))
        ),
        |((name, struct_name), fields)| Fieldset {
            name: name.to_string(),
            struct_name: struct_name.to_string(),
            fields: fields
        }
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
        assert_eq!(
            parse_fieldset(content),
            Ok(("", Fieldset {
                name: "PersonName".to_string(),
                struct_name: "Person".to_string(),
                fields: vec![],
            }))
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
        assert_eq!(
            parse_fieldset(content),
            Ok(("", Fieldset {
                name: "PersonName".to_string(),
                struct_name: "Person".to_string(),
                fields: vec![
                    Field { name: "name".to_string(), optional: false },
                ],
            }))
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
        "fieldset PersonName for Person{name,age? }",
        "fieldset PersonName for Person{name, age? }",
        "fieldset PersonName for Person { name, age? }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_fieldset(content),
            Ok(("", Fieldset {
                name: "PersonName".to_string(),
                struct_name: "Person".to_string(),
                fields: vec![
                    Field { name: "name".to_string(), optional: false },
                    Field { name: "age".to_string(), optional: true },
                ],
            }))
        )
    }
}
