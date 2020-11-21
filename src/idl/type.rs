#[cfg(test)]
use crate::idl::common::assert_parse;
use crate::idl::common::{parse_field_separator, parse_identifier, trailing_comma, ws, Span};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::{many0, separated_list0},
    sequence::{preceded, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Type {
    Ref(TypeRef),
    Array(Box<Type>),
    Map(Box<Type>, Box<Type>),
}

#[derive(Debug, PartialEq)]
pub struct TypeRef {
    pub abs: bool,
    pub ns: Vec<String>,
    pub name: String,
    pub generics: Vec<Type>,
}

pub fn parse_none(input: Span) -> IResult<Span, Option<Type>> {
    map(tag("None"), |_| None)(input)
}

pub fn parse_type_ref(input: Span) -> IResult<Span, TypeRef> {
    map(
        tuple((
            map(opt(tag("::")), |r| r.is_some()),
            parse_identifier,
            many0(preceded(tag("::"), parse_identifier)),
            parse_generics,
        )),
        |(abs, path_first, mut path, generics)| {
            let (ns, name): (Vec<String>, String) = match path.pop() {
                Some(name) => {
                    path.insert(0, path_first);
                    (path, name)
                }
                None => (path, path_first),
            };
            TypeRef {
                abs,
                ns,
                name,
                generics,
            }
        },
    )(input)
}

fn parse_generics(input: Span) -> IResult<Span, Vec<Type>> {
    map(
        opt(preceded(
            preceded(ws, char('<')),
            cut(terminated(
                separated_list0(parse_field_separator, preceded(ws, parse_type)),
                preceded(trailing_comma, preceded(ws, char('>'))),
            )),
        )),
        |v| match v {
            Some(v) => v,
            None => Vec::with_capacity(0),
        },
    )(input)
}

fn parse_type_array(input: Span) -> IResult<Span, Type> {
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

fn parse_type_map_inner(input: Span) -> IResult<Span, Type> {
    map(
        separated_pair(
            preceded(ws, parse_type),
            cut(preceded(ws, char(':'))),
            preceded(ws, parse_type),
        ),
        |t| Type::Map(Box::new(t.0), Box::new(t.1)),
    )(input)
}

fn parse_type_map(input: Span) -> IResult<Span, Type> {
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

pub fn parse_opt_type(input: Span) -> IResult<Span, Option<Type>> {
    preceded(ws, alt((parse_none, map(parse_type, Some))))(input)
}

pub fn parse_type(input: Span) -> IResult<Span, Type> {
    preceded(
        ws,
        alt((
            map(parse_type_ref, Type::Ref),
            parse_type_array,
            parse_type_map,
        )),
    )(input)
}

#[test]
fn test_parse_none() {
    assert_parse(parse_opt_type(Span::new("None")), None);
}

#[test]
fn test_parse_type_ref_rel_without_ns() {
    assert_parse(
        parse_type(Span::new("T")),
        Type::Ref(TypeRef {
            abs: false,
            ns: vec![],
            name: "T".to_string(),
            generics: vec![],
        }),
    );
}

#[test]
fn test_parse_type_ref_abs_without_ns() {
    assert_parse(
        parse_type(Span::new("::T")),
        Type::Ref(TypeRef {
            abs: true,
            ns: vec![],
            name: "T".to_string(),
            generics: vec![],
        }),
    );
}

#[test]
fn test_parse_type_ref_rel_with_ns() {
    assert_parse(
        parse_type(Span::new("ns::T")),
        Type::Ref(TypeRef {
            abs: false,
            ns: vec!["ns".to_string()],
            name: "T".to_string(),
            generics: vec![],
        }),
    );
}

#[test]
fn test_parse_type_ref_abs_with_ns() {
    assert_parse(
        parse_type(Span::new("::ns::T")),
        Type::Ref(TypeRef {
            abs: true,
            ns: vec!["ns".to_string()],
            name: "T".to_string(),
            generics: vec![],
        }),
    );
}

#[test]
fn test_parse_type_ref_rel_with_ns2() {
    assert_parse(
        parse_type(Span::new("ns1::ns2::T")),
        Type::Ref(TypeRef {
            abs: false,
            ns: vec!["ns1".to_string(), "ns2".to_string()],
            name: "T".to_string(),
            generics: vec![],
        }),
    );
}

#[test]
fn test_parse_type_ref_abs_with_ns2() {
    assert_parse(
        parse_type(Span::new("::ns1::ns2::T")),
        Type::Ref(TypeRef {
            abs: true,
            ns: vec!["ns1".to_string(), "ns2".to_string()],
            name: "T".to_string(),
            generics: vec![],
        }),
    );
}

#[test]
fn test_parse_type_ref_with_generic_ref() {
    let contents = [
        "Foo<UUID>",
        "Foo <UUID>",
        "Foo< UUID>",
        "Foo<UUID >",
        "Foo<UUID,>",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_type(Span::new(content)),
            Type::Ref(TypeRef {
                abs: false,
                ns: vec![],
                name: "Foo".to_string(),
                generics: vec![Type::Ref(TypeRef {
                    abs: false,
                    name: "UUID".to_string(),
                    ns: vec![],
                    generics: vec![],
                })],
            }),
        );
    }
}

#[test]
fn test_parse_type_ref_with_generic_generic() {
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
        assert_parse(
            parse_type(Span::new(content)),
            Type::Ref(TypeRef {
                abs: false,
                ns: vec![],
                name: "Foo".to_string(),
                generics: vec![Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Bar".to_string(),
                    generics: vec![Type::Ref(TypeRef {
                        abs: false,
                        ns: vec![],
                        name: "UUID".to_string(),
                        generics: vec![],
                    })],
                })],
            }),
        );
    }
}

#[test]
fn test_parse_type_array() {
    let contents = ["[UUID]", "[ UUID]", "[UUID ]", "[ UUID ]"];
    for content in contents.iter() {
        assert_parse(
            parse_type(Span::new(content)),
            Type::Array(Box::new(Type::Ref(TypeRef {
                abs: false,
                ns: vec![],
                name: "UUID".to_string(),
                generics: vec![],
            }))),
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
        assert_parse(
            parse_type(Span::new(content)),
            Type::Map(
                Box::new(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "UUID".to_string(),
                    generics: vec![],
                })),
                Box::new(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "String".to_string(),
                    generics: vec![],
                })),
            ),
        );
    }
}
