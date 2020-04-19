use nom::{
    branch::alt,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list,
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::idl::common::{parse_identifier, parse_field_separator, trailing_comma, ws};

#[derive(Debug, PartialEq)]
pub enum Type {
    Named(String, Vec<Type>),
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
}

fn parse_type_named(input: &str) -> IResult<&str, Type> {
    map(
        pair(
            parse_identifier,
            parse_generics,
        ),
        |t| Type::Named(t.0, t.1)
    )(input)
}

fn parse_generics(input: &str) -> IResult<&str, Vec<Type>> {
    map(
        opt(
            preceded(
                preceded(ws, char('<')),
                cut(terminated(
                    separated_list(parse_field_separator, preceded(ws, parse_type)),
                    preceded(trailing_comma, preceded(ws, char('>'))),
                )),
            ),
        ),
        |v| match v {
            Some(v) => v,
            None => Vec::with_capacity(0),
        }
    )(input)
}

fn parse_type_array(input: &str) -> IResult<&str, Type> {
    context(
        "array",
        preceded(
            char('['),
            cut(terminated(
                preceded(ws, map(parse_type, |t| Type::Array(Box::new(t)))),
                preceded(ws, char(']')),
            )),
        ),
    )(input)
}

fn parse_type_map_inner(input: &str) -> IResult<&str, Type> {
    map(
        separated_pair(
            preceded(ws, parse_type),
            cut(preceded(ws, char(':'))),
            preceded(ws, parse_type),
        ),
        |t| Type::Map(
            Box::new(t.0),
            Box::new(t.1)
        ),
    )(input)
}

fn parse_type_map(input: &str) -> IResult<&str, Type> {
    context(
        "map",
        preceded(
            char('{'),
            cut(terminated(
                preceded(ws, parse_type_map_inner),
                preceded(ws, char('}')),
            )),
        ),
    )(input)
}

pub fn parse_type(input: &str) -> IResult<&str, Type> {
    preceded(
        ws,
        alt((
            parse_type_named,
            parse_type_array,
            parse_type_map,
        )),
    )(input)
}

#[test]
fn test_parse_type_named() {
    let contents = ["Foo"];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Named("Foo".to_string(), vec![])))
        );
    }
}

#[test]
fn test_parse_type_named_with_generic_named() {
    let contents = [
        "Foo<UUID>",
        "Foo <UUID>",
        "Foo< UUID>",
        "Foo<UUID >",
        "Foo<UUID,>",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Named("Foo".to_string(), vec![Type::Named("UUID".to_string(), vec![])])))
        );
    }
}

#[test]
fn test_parse_type_named_with_generic_generic() {
    let contents = [
        "Foo<Bar<UUID>>",
        "Foo <Bar<UUID>>",
        "Foo< Bar<UUID>>",
        "Foo<Bar <UUID>>",
        "Foo<Bar< UUID>>",
        "Foo<Bar<UUID >>",
        "Foo<Bar<UUID> >",
        "Foo<Bar<UUID,>,>",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Named("Foo".to_string(), vec![
                Type::Named("Bar".to_string(), vec![
                    Type::Named("UUID".to_string(), vec![])
                ])
            ])))
        );
    }
}


#[test]
fn test_parse_type_array() {
    let contents = ["[UUID]", "[ UUID]", "[UUID ]", "[ UUID ]"];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Array(
                Box::new(Type::Named("UUID".to_string(), vec![]))
            )))
        );
    }
}

#[test]
fn test_parse_type_map() {
    let contents = [
        "{UUID:String}",
        "{ UUID:String}",
        "{UUID:String }",
        "{UUID :String}",
        "{UUID: String}",
        "{ UUID : String }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Map(
                Box::new(Type::Named("UUID".to_string(), vec![])),
                Box::new(Type::Named("String".to_string(), vec![]))
            )))
        );
    }
}
