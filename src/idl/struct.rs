use nom::{
    IResult,
    bytes::complete::{tag,},
    character::complete::char,
    combinator::{cut, map, opt},
    error::{context},
    multi::separated_list,
    sequence::{pair, preceded, separated_pair, terminated}
};

use crate::idl::common::{
    parse_identifier,
    parse_field_separator,
    trailing_comma,
    ws,
};
use crate::idl::r#type::{
    Type,
    parse_type,
};
use crate::idl::field_option:: {
    FieldOption,
    parse_field_options,
};

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: Type,
    pub optional: bool,
    pub options: Vec<FieldOption>,
}

pub fn parse_struct(input: &str) -> IResult<&str, Struct> {
    map(
        pair(
            preceded(
                tag("struct"),
                parse_identifier
            ),
            parse_fields
        ),
        |t| Struct {
            name: t.0.to_string(),
            fields: t.1
        }
    )(input)
}

fn parse_fields(input: &str) -> IResult<&str, Vec<Field>> {
    context(
        "fields",
        preceded(
            preceded(ws, char('{')),
            cut(terminated(
                separated_list(parse_field_separator, parse_field),
                preceded(trailing_comma, preceded(ws, char('}')))
            ))
        )
    )(input)
}

fn parse_field(input: &str) -> IResult<&str, Field> {
    map(
        separated_pair(
            pair(
                parse_identifier,
                opt(preceded(ws, char('?')))
            ),
            preceded(ws, char(':')),
            pair(
                parse_type,
                opt(parse_field_options),
            )
        ),
        |((name, optional), (type_, options))| Field {
            name: name,
            optional: optional != None,
            type_: type_,
            options: if let Some(options) = options { options } else { vec![] },
        }
    )(input)
}

#[test]
fn test_parse_field() {
    let contents = [
        "foo:FooType",
        "foo: FooType",
        "foo : FooType",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_field(content),
            Ok(("", Field {
                name: "foo".to_string(),
                type_: Type::Named("FooType".to_string()),
                optional: false,
                options: vec![],
            }))
        );
    }
}

#[test]
fn test_parse_field_optional() {
    let contents = [
        "foo?:FooType",
        "foo? :FooType",
        "foo ?:FooType",
        "foo ? :FooType",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_field(content),
            Ok(("", Field {
                name: "foo".to_string(),
                type_: Type::Named("FooType".to_string()),
                optional: true,
                options: vec![],
            }))
        );
    }
}

#[test]
fn test_parse_field_with_options() {
    use crate::idl::value::Value;
    let contents = [
        "name:String(length=2..50)",
        "name :String(length=2..50)",
        "name: String(length=2..50)",
        "name:String (length=2..50)",
        "name:String( length=2..50)",
        "name:String(length =2..50)",
        "name:String(length= 2..50)",
        /*
        "name:String(length=2 ..50)",
        "name:String(length=2.. 50)",
        */
        "name:String(length=2..50 )",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_field(content),
            Ok(("", Field {
                name: "name".to_string(),
                type_: Type::Named("String".to_string()),
                optional: false,
                options: vec![
                    FieldOption {
                        name: "length".to_string(),
                        value: Value::Range(Some(2), Some(50)),
                    }
                ],
            }))
        );
    }
}

#[test]
fn test_parse_array_field_with_options() {
    use crate::idl::value::Value;
    let contents = [
        "items:[String](length=0..32)",
        "items :[String](length=0..32)",
        "items: [String](length=0..32)",
        "items:[String] (length=0..32)",
        "items:[String]( length=0..32)",
        "items:[String](length =0..32)",
        "items:[String](length= 0..32)",
        "items:[String](length=0..32 )",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_field(content),
            Ok(("", Field {
                name: "items".to_string(),
                type_: Type::Array("String".to_string()),
                optional: false,
                options: vec![
                    FieldOption {
                        name: "length".to_string(),
                        value: Value::Range(Some(0), Some(32)),
                    }
                ],
            }))
        );
    }
}

#[test]
fn test_parse_fields_0() {
    let contents = [
        "{}",
        "{ }",
        "{,}",
        "{ ,}",
        "{, }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_fields(content),
            Ok(("", vec![]))
        );
    }
}

#[test]
fn test_parse_fields_1() {
    let contents = [
        "{foo:Foo}",
        "{foo: Foo}",
        "{foo:Foo }",
        "{ foo:Foo}",
        "{foo:Foo,}"
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_fields(content),
            Ok(("", vec![Field {
                name: "foo".to_owned(),
                type_: Type::Named("Foo".to_owned()),
                optional: false,
                options: vec![],
            }]))
        );
    }
}

#[test]
fn test_parse_fields_2() {
    let contents = [
        "{foo:Foo,bar:Bar}",
        "{foo: Foo, bar: Bar}",
        "{ foo:Foo,bar:Bar }",
        "{ foo: Foo, bar: Bar }",
        "{ foo: Foo, bar: Bar, }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_fields(content),
            Ok(("", vec![
                Field {
                    name: "foo".to_owned(),
                    type_: Type::Named("Foo".to_owned()),
                    optional: false,
                    options: vec![],
                },
                Field {
                    name: "bar".to_owned(),
                    type_: Type::Named("Bar".to_owned()),
                    optional: false,
                    options: vec![],
                }
            ]))
        );
    }
}

#[test]
fn test_parse_struct() {
    let contents = [
        "struct Pinger{}",
        "struct Pinger {}",
        "struct Pinger{ }",
        "struct Pinger { }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_struct(content),
            Ok(("", Struct {
                name: "Pinger".to_string(),
                fields: vec![],
            }))
        );
    }
}

#[test]
fn test_parse_struct_field_options() {
    use crate::idl::value::Value;
    let contents = [
        "struct Person { name: [String] (length=1..50) }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_struct(content),
            Ok(("", Struct {
                name: "Person".to_string(),
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        type_: Type::Array("String".to_string()),
                        optional: false,
                        options: vec![
                            FieldOption {
                                name: "length".to_string(),
                                value: Value::Range(Some(1), Some(50))
                            }
                        ]
                    }
                ],
            }))
        );
    }
}

#[test]
fn test_parse_struct_invalid() {
    use nom::error::ErrorKind;
    assert_eq!(
        parse_struct("struct 123fail{}"),
        Err(nom::Err::Error(("123fail{}", ErrorKind::TakeWhile1)))
    )
}

#[test]
fn test_parse_struct_with_fields() {
    let contents = [
        // no whitespace
        "struct Person {name:String,age:Integer}",
        // whitespace after colon
        "struct Person {name: String,age: Integer}",
        // whitespace after comma
        "struct Person {name:String, age:Integer}",
        // whitespace before comma
        "struct Person {name: String ,age:Integer}",
        // whitespace between braces
        "struct Person { name:String,age:Integer }",
        // trailing comma
        "struct Person {name:String,age:Integer,}",
        // trailing comma space after
        "struct Person {name:String,age:Integer, }",
        // trailing comma space before
        "struct Person {name:String,age:Integer ,}",
        // all combined
        "struct Person { name: String , age: Integer , }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_struct(content),
            Ok(("", Struct {
                name: "Person".to_string(),
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        type_: Type::Named("String".to_string()),
                        optional: false,
                        options: vec![],
                    },
                    Field {
                        name: "age".to_string(),
                        type_: Type::Named("Integer".to_string()),
                        optional: false,
                        options: vec![],
                    },
                ],
            }))
        )
    }
}
