use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alpha1, alphanumeric0, char},
    combinator::{cut, map, opt},
    error::{context, ErrorKind},
    multi::separated_list,
    sequence::{pair, preceded, separated_pair, terminated, tuple}
};

const WHITSPACE: &str = " \t\r\n";

#[derive(Debug, PartialEq)]
pub struct Enum {
    pub name: String,
    pub values: Vec<String>
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_: Type,
    pub optional: bool
}

#[derive(Debug, PartialEq)]
pub enum Type {
    Named(String),
    Array(String),
    Map(String, String),
}

#[derive(Debug, PartialEq)]
pub struct FieldsetField {
    pub name: String,
    pub optional: bool
}

#[derive(Debug, PartialEq)]
pub struct Fieldset {
    pub name: String,
    pub struct_name: String,
    pub fields: Vec<FieldsetField>,
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub request: String,
    pub response: String
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub name: String,
    // FIXME replace by in/out/err
    pub fields: Vec<StructField>,
}

#[derive(Debug, PartialEq)]
pub enum DocumentPart {
    Enum(Enum),
    Struct(Struct),
    Fieldset(Fieldset),
    Function(Function),
    Service(Service)
}

#[derive(Debug, PartialEq)]
pub struct Document {
    pub parts: Vec<DocumentPart>
}

fn ws(input: &str) -> IResult<&str, &str> {
    take_while(move |c| WHITSPACE.contains(c))(input)
}

fn ws1(input: &str) -> IResult<&str, &str> {
    take_while1(move |c| WHITSPACE.contains(c))(input)
}

fn trailing_comma(input: &str) -> IResult<&str, Option<char>> {
    opt(preceded(
        ws,
        char(',')
    ))(input)
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    preceded(
        ws,
        map(
            pair(alpha1, alphanumeric0),
            |t| format!("{}{}", t.0, t.1)
        )
    )(input)
}

fn parse_field(input: &str) -> IResult<&str, StructField> {
    map(
        separated_pair(
            pair(
                parse_identifier,
                opt(preceded(ws, char('?')))
            ),
            preceded(ws, char(':')),
            parse_type
        ),
        |((name, optional), type_)| StructField {
            name: name,
            optional: optional != None,
            type_: type_
        }
    )(input)
}

fn parse_type_array(input: &str) -> IResult<&str, Type> {
    context("array",
        preceded(
            char('['),
            cut(terminated(
                map(parse_identifier, Type::Array),
                preceded(ws, char(']'))
            ))
        )
    )(input)
}

fn parse_type_map_inner(input: &str) -> IResult<&str, Type> {
    map(
        separated_pair(
            parse_identifier,
            cut(preceded(ws, char(':'))),
            parse_identifier,
        ),
        |types| Type::Map(types.0.to_string(), types.1.to_string())
    )(input)
}

fn parse_type_map(input: &str) -> IResult<&str, Type> {
    context("map",
        preceded(
            char('{'),
            cut(terminated(
                preceded(ws, parse_type_map_inner),
                preceded(ws, char('}'))
            ))
        )
    )(input)
}

fn parse_type(input: &str) -> IResult<&str, Type> {
    alt((
        map(parse_identifier, Type::Named),
        parse_type_array,
        parse_type_map,
    ))(input)
}

#[test]
fn test_parse_type_array() {
    let contents = [
        "[UUID]",
        "[ UUID]",
        "[UUID ]",
        "[ UUID ]",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Type::Array("UUID".to_string())))
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
                "UUID".to_string(),
                "String".to_string()
            )))
        );
    }
}

fn parse_enum(input: &str) -> IResult<&str, Enum> {
    map(
        pair(
            preceded(
                tag("enum"),
                parse_identifier
            ),
            parse_enum_values
        ),
        |t| Enum {
            name: t.0.to_string(),
            values: t.1
        }
    )(input)
}

fn parse_enum_values(input: &str) -> IResult<&str, Vec<String>> {
    context(
        "enum_values",
        preceded(
            preceded(ws, char('{')),
            cut(terminated(
                separated_list(parse_field_separator, parse_identifier),
                preceded(trailing_comma, preceded(ws, char('}')))
            ))
        )
    )(input)
}

fn parse_field_separator(input: &str) -> IResult<&str, char> {
    preceded(ws, char(','))(input)
}

fn parse_struct_fields(input: &str) -> IResult<&str, Vec<StructField>> {
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

fn parse_struct(input: &str) -> IResult<&str, Struct> {
    map(
        pair(
            preceded(
                tag("struct"),
                parse_identifier
            ),
            parse_struct_fields
        ),
        |t| Struct {
            name: t.0.to_string(),
            fields: t.1
        }
    )(input)
}

fn parse_fieldset_field(input: &str) -> IResult<&str, FieldsetField> {
    map(
        pair(
            parse_identifier,
            opt(char('?'))
        ),
        |(name, optional)| FieldsetField {
            name: name.to_string(),
            optional: optional != None
        }
    )(input)
}

fn parse_fieldset_fields(input: &str) -> IResult<&str, Vec<FieldsetField>> {
    preceded(
        preceded(ws, char('{')),
        cut(terminated(
            separated_list(parse_field_separator, parse_fieldset_field),
            preceded(trailing_comma, preceded(ws, char('}')))
        ))
    )(input)
}

fn parse_fieldset(input: &str) -> IResult<&str, Fieldset> {
    map(
        preceded(
            terminated(tag("fieldset"), ws1),
            cut(pair(
                separated_pair(
                    parse_identifier,
                    preceded(ws, tag("for")),
                    preceded(ws1, parse_identifier),
                ),
                parse_fieldset_fields
            ))
        ),
        |((name, struct_name), fields)| Fieldset {
            name: name.to_string(),
            struct_name: struct_name.to_string(),
            fields: fields
        }
    )(input)
}

fn parse_service(input: &str) -> IResult<&str, Service> {
    map(
        preceded(
            terminated(tag("service"), ws1),
            cut(pair(
                parse_identifier,
                parse_struct_fields,
            ))
        ),
        |(name, fields)| Service {
            name: name,
            fields: fields
        }
    )(input)
}

fn parse_document_part(input: &str) -> IResult<&str, DocumentPart> {
    alt((
        map(parse_enum, DocumentPart::Enum),
        map(parse_struct, DocumentPart::Struct),
        map(parse_fieldset, DocumentPart::Fieldset),
        map(parse_service, DocumentPart::Service),
        // TODO add support for functions
    ))(input)
}

pub fn parse_document(input: &str) -> IResult<&str, Document> {
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, parts) = separated_list(take_while1(char::is_whitespace), parse_document_part)(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    // FIXME fail if there is remaining input
    Ok((input, Document {
        parts: parts
    }))
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
    assert_eq!(
        parse_identifier("123test"),
        Err(nom::Err::Error(("123test", ErrorKind::Alpha)))
    );
    assert_eq!(
        parse_identifier("_test"),
        Err(nom::Err::Error(("_test", ErrorKind::Alpha)))
    );
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
            Ok(("", StructField {
                name: "foo".to_string(),
                type_: Type::Named("FooType".to_string()),
                optional: false
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
            Ok(("", StructField {
                name: "foo".to_string(),
                type_: Type::Named("FooType".to_string()),
                optional: true
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
            parse_struct_fields(content),
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
            parse_struct_fields(content),
            Ok(("", vec![StructField {
                name: "foo".to_owned(),
                type_: Type::Named("Foo".to_owned()),
                optional: false
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
            parse_struct_fields(content),
            Ok(("", vec![
                StructField {
                    name: "foo".to_owned(),
                    type_: Type::Named("Foo".to_owned()),
                    optional: false
                },
                StructField {
                    name: "bar".to_owned(),
                    type_: Type::Named("Bar".to_owned()),
                    optional: false
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
fn test_parse_struct_invalid() {
    assert_eq!(
        parse_struct("struct 123fail{}"),
        Err(nom::Err::Error(("123fail{}", ErrorKind::Alpha)))
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
                    StructField { name: "name".to_string(), type_: Type::Named("String".to_string()), optional: false },
                    StructField { name: "age".to_string(), type_: Type::Named("Integer".to_string()), optional: false },
                ],
            }))
        )
    }
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
                    FieldsetField { name: "name".to_string(), optional: false },
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
                    FieldsetField { name: "name".to_string(), optional: false },
                    FieldsetField { name: "age".to_string(), optional: true },
                ],
            }))
        )
    }
}

#[test]
fn test_parse_service_with_fields() {
    let contents = [
        // no whitespace
        "service Pinger {request:Ping,response:Pong}",
        // whitespace after colon
        "service Pinger {request: Ping,response: Pong}",
        // whitespace after comma
        "service Pinger {request:Ping, response:Pong}",
        // whitespace before comma
        "service Pinger {request:Ping ,response:Pong}",
        // whitespace between braces
        "service Pinger { request:Ping,response:Pong }",
        // trailing comma
        "service Pinger {request:Ping,response:Pong,}",
        // trailing comma space after
        "service Pinger {request: Ping,response:Pong, }",
        // trailing comma space before
        "service Pinger {request: Ping,response:Pong ,}",
        // all combined
        "service Pinger { request: Ping , response: Pong , }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_service(content),
            Ok(("", Service {
                name: "Pinger".to_string(),
                fields: vec![
                    StructField { name: "request".to_string(), type_: Type::Named("Ping".to_string()), optional: false },
                    StructField { name: "response".to_string(), type_: Type::Named("Pong".to_string()), optional: false },
                ],
            }))
        )
    }
}

#[test]
fn test_parse_document() {
    let content = "
        struct Person {
            name:String,
            age:Integer
        }
        struct Group {
            name:String
        }
        service Pinger {
            request:Ping,
            response:Pong
        }
    ";
    assert_eq!(
        parse_document(content),
        Ok(("", Document {
            parts: vec![
                DocumentPart::Struct(Struct {
                    name: "Person".to_string(),
                    fields: vec![
                        StructField { name: "name".to_string(), type_: Type::Named("String".to_string()), optional: false },
                        StructField { name: "age".to_string(), type_: Type::Named("Integer".to_string()), optional: false },
                    ],
                }),
                DocumentPart::Struct(Struct {
                    name: "Group".to_string(),
                    fields: vec![
                        StructField { name: "name".to_string(), type_: Type::Named("String".to_string()), optional: false },
                    ],
                }),
                DocumentPart::Service(Service {
                    name: "Pinger".to_string(),
                    fields: vec![
                        StructField { name: "request".to_string(), type_: Type::Named("Ping".to_string()), optional: false },
                        StructField { name: "response".to_string(), type_: Type::Named("Pong".to_string()), optional: false },
                    ],
                }),
            ]
        }))
    )
}
