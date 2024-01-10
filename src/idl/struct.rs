use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, map, opt},
    error::context,
    multi::separated_list0,
    sequence::{pair, preceded, separated_pair, terminated},
    IResult,
};

use crate::common::FilePosition;
use crate::idl::common::{
    parse_field_separator, parse_identifier, parse_identifier_with_generics, trailing_comma, ws,
    ws1, Span,
};
use crate::idl::field_option::{parse_field_options, FieldOption};
use crate::idl::r#type::{parse_type, Type};

#[cfg(test)]
use crate::idl::common::assert_parse;
#[cfg(test)]
use nom::Slice;

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<Field>,
    pub position: FilePosition,
}

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: Type,
    pub optional: bool,
    pub options: Vec<FieldOption>,
    pub position: FilePosition,
}

pub fn parse_struct(input: Span) -> IResult<Span, Struct> {
    map(
        pair(
            preceded(tag("struct"), preceded(ws1, parse_identifier_with_generics)),
            parse_fields,
        ),
        |((name, generics), fields)| Struct {
            name,
            generics,
            fields,
            position: input.into(),
        },
    )(input)
}

fn parse_fields(input: Span) -> IResult<Span, Vec<Field>> {
    context(
        "fields",
        preceded(
            preceded(ws, char('{')),
            cut(terminated(
                separated_list0(parse_field_separator, preceded(ws, parse_field)),
                preceded(trailing_comma, preceded(ws, char('}'))),
            )),
        ),
    )(input)
}

fn parse_field(input: Span) -> IResult<Span, Field> {
    map(
        separated_pair(
            pair(parse_identifier, opt(preceded(ws, char('?')))),
            preceded(ws, char(':')),
            pair(parse_type, opt(parse_field_options)),
        ),
        |((name, optional), (type_, options))| Field {
            name,
            position: input.into(),
            optional: optional.is_some(),
            type_,
            options: if let Some(options) = options {
                options
            } else {
                vec![]
            },
        },
    )(input)
}

#[test]
fn test_parse_field() {
    use crate::idl::r#type::TypeRef;
    let contents = ["foo:FooType", "foo: FooType", "foo : FooType"];
    for content in contents.iter() {
        assert_parse(
            parse_field(Span::new(content)),
            Field {
                name: "foo".to_string(),
                position: FilePosition { line: 1, column: 1 },
                type_: Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "FooType".to_string(),
                    generics: vec![],
                }),
                optional: false,
                options: vec![],
            },
        );
    }
}

#[test]
fn test_parse_field_optional() {
    use crate::idl::r#type::TypeRef;
    let contents = [
        "foo?:FooType",
        "foo? :FooType",
        "foo ?:FooType",
        "foo ? :FooType",
    ];
    for content in contents.iter() {
        assert_parse(
            parse_field(Span::new(content)),
            Field {
                name: "foo".to_string(),
                position: FilePosition { line: 1, column: 1 },
                type_: Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "FooType".to_string(),
                    generics: vec![],
                }),
                optional: true,
                options: vec![],
            },
        );
    }
}

#[test]
fn test_parse_field_with_options() {
    use crate::idl::r#type::TypeRef;
    use crate::idl::value::Value;
    let contents = [
        ("name:String(length=2..50)", 13),
        ("name :String(length=2..50)", 14),
        ("name: String(length=2..50)", 14),
        ("name:String (length=2..50)", 14),
        ("name:String( length=2..50)", 14),
        ("name:String(length =2..50)", 13),
        ("name:String(length= 2..50)", 13),
        /*
        "(name:String(length=2 ..50)", 13),
        "(name:String(length=2.. 50)", 13),
        */
        ("name:String(length=2..50 )", 13),
    ];
    for (content, length_column) in contents.iter() {
        assert_parse(
            parse_field(Span::new(content)),
            Field {
                name: "name".to_string(),
                position: FilePosition { line: 1, column: 1 },
                type_: Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "String".to_string(),
                    generics: vec![],
                }),
                optional: false,
                options: vec![FieldOption {
                    position: FilePosition {
                        line: 1,
                        column: *length_column,
                    },
                    name: "length".to_string(),
                    value: Value::Range(Some(2), Some(50)),
                }],
            },
        );
    }
}

#[test]
fn test_parse_array_field_with_options() {
    use crate::idl::r#type::TypeRef;
    use crate::idl::value::Value;
    let contents = [
        ("items:[String](length=0..32)", 16),
        ("items :[String](length=0..32)", 17),
        ("items: [String](length=0..32)", 17),
        ("items:[String] (length=0..32)", 17),
        ("items:[String]( length=0..32)", 17),
        ("items:[String](length =0..32)", 16),
        ("items:[String](length= 0..32)", 16),
        ("items:[String](length=0..32 )", 16),
    ];
    for (content, length_column) in contents.iter() {
        assert_parse(
            parse_field(Span::new(content)),
            Field {
                name: "items".to_string(),
                position: FilePosition { line: 1, column: 1 },
                type_: Type::Array(Box::new(Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "String".to_string(),
                    generics: vec![],
                }))),
                optional: false,
                options: vec![FieldOption {
                    position: FilePosition {
                        line: 1,
                        column: *length_column,
                    },
                    name: "length".to_string(),
                    value: Value::Range(Some(0), Some(32)),
                }],
            },
        );
    }
}

#[test]
fn test_parse_fields_0() {
    let contents = ["{}", "{ }", "{,}", "{ ,}", "{, }"];
    for content in contents.iter() {
        assert_parse(parse_fields(Span::new(content)), vec![])
    }
}

#[test]
fn test_parse_fields_1() {
    use crate::idl::r#type::TypeRef;
    let content = "{foo: Foo}";
    assert_parse(
        parse_fields(Span::new(content)),
        vec![Field {
            name: "foo".to_owned(),
            position: FilePosition { line: 1, column: 2 },
            type_: Type::Ref(TypeRef {
                abs: false,
                ns: vec![],
                name: "Foo".to_owned(),
                generics: vec![],
            }),
            optional: false,
            options: vec![],
        }],
    );
}

#[test]
fn test_parse_fields_1_ws_variants() {
    let contents = ["{foo: Foo}", "{foo:Foo }", "{ foo:Foo}", "{foo:Foo,}"];
    for content in contents.iter() {
        let (_, f) = parse_fields(Span::new(content)).unwrap();
        assert_eq!(f.len(), 1);
    }
}

#[test]
fn test_parse_fields_2() {
    use crate::idl::r#type::TypeRef;
    let content = "{ foo: Foo, bar: Bar }";
    assert_parse(
        parse_fields(Span::new(content)),
        vec![
            Field {
                name: "foo".to_owned(),
                position: FilePosition { line: 1, column: 3 },
                type_: Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Foo".to_owned(),
                    generics: vec![],
                }),
                optional: false,
                options: vec![],
            },
            Field {
                name: "bar".to_owned(),
                position: FilePosition {
                    line: 1,
                    column: 13,
                },
                type_: Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "Bar".to_owned(),
                    generics: vec![],
                }),
                optional: false,
                options: vec![],
            },
        ],
    );
}

#[test]
fn test_parse_fields_2_ws_variants() {
    let contents = [
        "{foo:Foo,bar:Bar}",
        "{ foo:Foo,bar:Bar}",
        "{foo :Foo,bar:Bar}",
        "{foo: Foo,bar:Bar}",
        "{foo:Foo ,bar:Bar}",
        "{foo:Foo, bar:Bar}",
        "{foo:Foo,bar :Bar}",
        "{foo:Foo,bar: Bar}",
        "{foo:Foo,bar:Bar }",
        // trailing comma
        "{foo:Foo,bar:Bar,}",
    ];
    for content in contents.iter() {
        let (_, f) = parse_fields(Span::new(content)).unwrap();
        assert_eq!(f.len(), 2);
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
        assert_parse(
            parse_struct(Span::new(content)),
            Struct {
                name: "Pinger".to_string(),
                position: FilePosition { line: 1, column: 1 },
                generics: vec![],
                fields: vec![],
            },
        );
    }
}

#[test]
fn test_parse_struct_field_options() {
    use crate::idl::r#type::TypeRef;
    use crate::idl::value::Value;
    let contents = ["struct Person { name: [String] (length=1..50) }"];
    for content in contents.iter() {
        assert_parse(
            parse_struct(Span::new(content)),
            Struct {
                name: "Person".to_string(),
                position: FilePosition { line: 1, column: 1 },
                generics: vec![],
                fields: vec![Field {
                    name: "name".to_string(),
                    position: FilePosition {
                        line: 1,
                        column: 17,
                    },
                    type_: Type::Array(Box::new(Type::Ref(TypeRef {
                        abs: false,
                        ns: vec![],
                        name: "String".to_string(),
                        generics: vec![],
                    }))),
                    optional: false,
                    options: vec![FieldOption {
                        position: FilePosition {
                            line: 1,
                            column: 33,
                        },
                        name: "length".to_string(),
                        value: Value::Range(Some(1), Some(50)),
                    }],
                }],
            },
        );
    }
}

#[test]
fn test_parse_struct_invalid() {
    use nom::error::ErrorKind;
    let input = Span::new("struct 123fail{}");
    assert_eq!(
        parse_struct(input),
        // FIXME the error position is probably incorrect
        Err(nom::Err::Error(nom::error::Error {
            input: input.slice(7..),
            code: ErrorKind::TakeWhile1
        }))
    )
}

#[test]
fn test_parse_struct_with_fields() {
    use crate::idl::r#type::TypeRef;
    let contents = ["struct Person { name: String, age: Integer }"];
    for content in contents.iter() {
        assert_parse(
            parse_struct(Span::new(content)),
            Struct {
                name: "Person".to_string(),
                position: FilePosition { line: 1, column: 1 },
                generics: vec![],
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        position: FilePosition {
                            line: 1,
                            column: 17,
                        },
                        type_: Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "String".to_string(),
                            generics: vec![],
                        }),
                        optional: false,
                        options: vec![],
                    },
                    Field {
                        name: "age".to_string(),
                        position: FilePosition {
                            line: 1,
                            column: 31,
                        },
                        type_: Type::Ref(TypeRef {
                            abs: false,
                            ns: vec![],
                            name: "Integer".to_string(),
                            generics: vec![],
                        }),
                        optional: false,
                        options: vec![],
                    },
                ],
            },
        )
    }
}

#[test]
fn test_parse_struct_with_fields_ws_variants() {
    let contents = [
        "struct Person{name:String,age:Integer}",
        "struct Person {name:String,age:Integer}",
        "struct Person{ name:String,age:Integer}",
        "struct Person{name :String,age:Integer}",
        "struct Person{name: String,age:Integer}",
        "struct Person{name:String ,age:Integer}",
        "struct Person{name:String, age:Integer}",
        "struct Person{name:String,age :Integer}",
        "struct Person{name:String,age: Integer}",
        "struct Person{name:String,age:Integer }",
    ];
    for content in contents.iter() {
        let (_, s) = parse_struct(Span::new(content)).unwrap();
        assert_eq!(s.name, "Person");
        assert_eq!(s.fields.len(), 2);
    }
}

#[test]
fn test_parse_struct_with_generics() {
    use crate::idl::r#type::TypeRef;
    let content = "struct Wrapper<T> { value:T }";
    assert_parse(
        parse_struct(Span::new(content)),
        Struct {
            name: "Wrapper".to_string(),
            position: FilePosition { line: 1, column: 1 },
            generics: vec!["T".to_string()],
            fields: vec![Field {
                name: "value".to_string(),
                position: FilePosition {
                    line: 1,
                    column: 21,
                },
                type_: Type::Ref(TypeRef {
                    abs: false,
                    ns: vec![],
                    name: "T".to_string(),
                    generics: vec![],
                }),
                optional: false,
                options: vec![],
            }],
        },
    );
}

#[test]
fn test_parse_struct_with_generics_ws_variants() {
    let contents = [
        "struct Wrapper<T>{value:T}",
        "struct Wrapper <T>{value:T}",
        "struct Wrapper< T>{value:T}",
        "struct Wrapper<T >{value:T}",
        "struct Wrapper<T> {value:T}",
        "struct Wrapper<T>{ value:T}",
        "struct Wrapper<T>{value :T}",
        "struct Wrapper<T>{value: T}",
        "struct Wrapper<T>{value:T }",
        "struct Wrapper<T,>{value:T}",
        "struct Wrapper<T,>{value:T,}",
    ];
    for content in contents.iter() {
        let (_, s) = parse_struct(Span::new(content)).unwrap();
        assert_eq!(s.name, "Wrapper");
        assert_eq!(s.fields.len(), 1);
    }
}
