use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list0,
    sequence::{pair, preceded, terminated, tuple},
    IResult,
};

use crate::common::FilePosition;
use crate::idl::common::{
    parse_field_separator, parse_identifier, parse_identifier_with_generics, trailing_comma, ws,
    ws1, Span,
};
use crate::idl::r#type::{parse_type, Type};

#[cfg(test)]
use crate::idl::common::assert_parse;

use super::{r#type::parse_type_ref, TypeRef};

#[derive(Debug, PartialEq)]
pub struct Enum {
    pub name: String,
    pub generics: Vec<String>,
    pub extends: Option<TypeRef>,
    pub variants: Vec<EnumVariant>,
    pub position: FilePosition,
}

#[derive(Debug, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value_type: Option<Type>,
}

pub fn parse_enum(input: Span) -> IResult<Span, Enum> {
    map(
        tuple((
            preceded(terminated(tag("enum"), ws1), parse_identifier_with_generics),
            parse_enum_extends,
            parse_enum_variants,
        )),
        |((name, generics), extends, variants)| Enum {
            name,
            generics,
            extends,
            variants,
            position: input.into(),
        },
    )(input)
}

fn parse_enum_extends(input: Span) -> IResult<Span, Option<TypeRef>> {
    context(
        "enum_extends",
        opt(preceded(
            terminated(preceded(ws1, tag("extends")), ws1),
            parse_type_ref,
        )),
    )(input)
}

fn parse_enum_variants(input: Span) -> IResult<Span, Vec<EnumVariant>> {
    context(
        "enum_variants",
        preceded(
            preceded(ws, char('{')),
            cut(terminated(
                separated_list0(parse_field_separator, preceded(ws, parse_enum_variant)),
                preceded(trailing_comma, preceded(ws, char('}'))),
            )),
        ),
    )(input)
}

fn parse_enum_variant(input: Span) -> IResult<Span, EnumVariant> {
    context(
        "enum_variant",
        map(
            pair(
                parse_identifier,
                opt(preceded(
                    preceded(ws, char('(')),
                    cut(terminated(
                        parse_type,
                        preceded(trailing_comma, preceded(ws, char(')'))),
                    )),
                )),
            ),
            |(name, value_type)| EnumVariant { name, value_type },
        ),
    )(input)
}

#[test]
fn test_parse_enum_0() {
    let contents = [
        // minimal whitespace
        "enum Nothing{}",
        // normal whitespace
        "enum Nothing {}",
        // whitespace variants
        "enum Nothing { }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_enum(Span::new(content)),
            Enum {
                name: "Nothing".to_string(),
                generics: vec![],
                position: FilePosition { line: 1, column: 1 },
                extends: None,
                variants: vec![],
            },
        )
    }
}

#[test]
fn test_parse_enum_1() {
    let contents = [
        // minimal whitespace
        "enum OneThing{Thing}",
        // whitespace variants
        "enum OneThing {Thing}",
        "enum OneThing{ Thing}",
        "enum OneThing{Thing }",
        "enum OneThing { Thing }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_enum(Span::new(content)),
            Enum {
                name: "OneThing".to_string(),
                generics: vec![],
                position: FilePosition { line: 1, column: 1 },
                extends: None,
                variants: vec![EnumVariant {
                    name: "Thing".to_string(),
                    value_type: None,
                }],
            },
        )
    }
}

#[test]
fn test_parse_enum_2() {
    let contents = [
        // minimal whitespace
        "enum Direction{Left,Right}",
        // normal whitespace
        "enum Direction { Left, Right }",
        // whitespace variants
        "enum Direction {Left,Right}",
        "enum Direction{ Left,Right}",
        "enum Direction{Left ,Right}",
        "enum Direction{Left, Right}",
        "enum Direction{Left,Right }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_enum(Span::new(content)),
            Enum {
                name: "Direction".to_string(),
                generics: vec![],
                position: FilePosition { line: 1, column: 1 },
                extends: None,
                variants: vec![
                    EnumVariant {
                        name: "Left".to_string(),
                        value_type: None,
                    },
                    EnumVariant {
                        name: "Right".to_string(),
                        value_type: None,
                    },
                ],
            },
        )
    }
}

#[test]
fn test_parse_enum_with_value() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        // minimal whitespace
        "enum Value{S(String),I(Integer)}",
        // normal whitespace
        "enum Value { S(String), I(Integer) }",
        // whitespace variants
        "enum Value {S(String),I(Integer)}",
        "enum Value{ S(String),I(Integer)}",
        "enum Value{S (String),I(Integer)}",
        "enum Value{S( String),I(Integer)}",
        "enum Value{S(String ),I(Integer)}",
        "enum Value{S(String) ,I(Integer)}",
        "enum Value{S(String), I(Integer)}",
        "enum Value{S(String),I (Integer)}",
        "enum Value{S(String),I( Integer)}",
        "enum Value{S(String),I(Integer )}",
        "enum Value{S(String),I(Integer) }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_enum(Span::new(content)),
            Enum {
                name: "Value".to_string(),
                generics: vec![],
                position: FilePosition { line: 1, column: 1 },
                extends: None,
                variants: vec![
                    EnumVariant {
                        name: "S".to_string(),
                        value_type: Some(Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "String".to_string(),
                            generics: vec![],
                        })),
                    },
                    EnumVariant {
                        name: "I".to_string(),
                        value_type: Some(Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "Integer".to_string(),
                            generics: vec![],
                        })),
                    },
                ],
            },
        )
    }
}

#[test]
fn test_parse_enum_extends() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        // minimal whitespace
        "enum GetError extends GenericError{}",
        // normal whitespace
        "enum GetError extends GenericError {}",
        // whitespace variants
        "enum GetError extends GenericError{ }",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_enum(Span::new(content)),
            Enum {
                name: "GetError".to_string(),
                generics: vec![],
                position: FilePosition { line: 1, column: 1 },
                extends: Some(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "GenericError".to_string(),
                    generics: vec![],
                }),
                variants: vec![],
            },
        )
    }
}
