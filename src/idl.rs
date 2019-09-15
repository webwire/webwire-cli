use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{
        alpha1,
        alphanumeric0,
    },
    combinator::{
        opt,
    },
    error::ErrorKind,
    multi::separated_list
};

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: String,
    pub optional: bool
}

#[derive(Debug, PartialEq)]
pub struct Struct {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub struct FieldsetField {
    pub name: String,
    pub optional: bool
}

#[derive(Debug, PartialEq)]
pub struct Fieldset {
    pub name: String,
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
    // FIXME
    pub fields: Vec<Field>,
}

#[derive(Debug, PartialEq)]
pub enum DocumentPart {
    Struct(Struct),
    Fieldset(Fieldset),
    Function(Function),
    Service(Service)
}

#[derive(Debug, PartialEq)]
pub struct Document {
    pub parts: Vec<DocumentPart>
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    let (input, first) = alpha1(input)?;
    let (input, rest) = alphanumeric0(input)?;
    Ok((input, String::from(first) + rest))
}

fn parse_field(input: &str) -> IResult<&str, Field> {
    let (input, name) = parse_identifier(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, optional) = opt(tag("?"))(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, type_) = parse_identifier(input)?;
    Ok((input, Field {
        name: name.to_string(),
        type_: type_.to_string(),
        optional: optional != None
    }))
}

fn parse_field_separator(input: &str) -> IResult<&str, ()> {
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    Ok((input, ()))
}

fn parse_fields(input: &str) -> IResult<&str, Vec<Field>> {
    let (input, _) = tag("{")(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, fields) = separated_list(parse_field_separator, parse_field)(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, _) = opt(tag(","))(input)?; // trailing comma
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, _) = tag("}")(input)?;
    Ok((input, fields))
}

fn parse_type(input: &str) -> IResult<&str, Struct> {
    let (input, _) = tag("type")(input)?;
    let (input, _) = take_while1(char::is_whitespace)(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, fields) = parse_fields(input)?;
    Ok((input, Struct {
        name: name.to_string(),
        fields: fields
    }))
}

fn parse_service(input: &str) -> IResult<&str, Service> {
    let (input, _) = tag("service")(input)?;
    let (input, _) = take_while1(char::is_whitespace)(input)?;
    let (input, name) = parse_identifier(input)?;
    let (input, _) = take_while(char::is_whitespace)(input)?;
    let (input, fields) = parse_fields(input)?;
    Ok((input, Service {
        name: name.to_string(),
        fields: fields
    }))
}

/* FIXME
pub fn parse_document_part(input: &str) -> IResult<&str, DocumentPart> {
    let (input, part) = alt((
        parse_struct,
        parse_fieldset,
    ))(input);
    Ok((input, part))
}

pub fn parse_document(input: &str) -> IResult<&str, Document> {
    let (input, values) = many0(
        alt(
            parse_struct,
            // TODO parse function
            // TODO parse
            parse_service,
        )
    )
    (Document {
    })
}
*/

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
            Ok(("", Field {
                name: "foo".to_string(),
                type_: "FooType".to_string(),
                optional: false
            }))
        );
    }
}

#[test]
fn test_parse_optional_field() {
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
                type_: "FooType".to_string(),
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
                type_: "Foo".to_owned(),
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
            parse_fields(content),
            Ok(("", vec![
                Field {
                    name: "foo".to_owned(),
                    type_: "Foo".to_owned(),
                    optional: false
                },
                Field {
                    name: "bar".to_owned(),
                    type_: "Bar".to_owned(),
                    optional: false
                }
            ]))
        );
    }
}

#[test]
fn test_parse_type() {
    let contents = [
        "type Pinger{}",
        "type Pinger {}",
        "type Pinger{ }",
        "type Pinger { }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Struct {
                name: "Pinger".to_string(),
                fields: vec![],
            }))
        );
    }
}

#[test]
fn test_parse_type_invalid() {
    assert_eq!(
        parse_type("type 123fail{}"),
        Err(nom::Err::Error(("123fail{}", ErrorKind::Alpha)))
    )
}

#[test]
fn test_parse_type_with_fields() {
    let contents = [
        // no whitespace
        "type Person {name:String,age:Integer}",
        // whitespace after colon
        "type Person {name: String,age: Integer}",
        // whitespace after comma
        "type Person {name:String, age:Integer}",
        // whitespace before comma
        "type Person {name: String ,age:Integer}",
        // whitespace between braces
        "type Person { name:String,age:Integer }",
        // trailing comma
        "type Person {name:String,age:Integer,}",
        // trailing comma space after
        "type Person {name:String,age:Integer, }",
        // trailing comma space before
        "type Person {name:String,age:Integer ,}",
        // all combined
        "type Person { name: String , age: Integer , }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_type(content),
            Ok(("", Struct {
                name: "Person".to_string(),
                fields: vec![
                    Field { name: "name".to_string(), type_: "String".to_string(), optional: false },
                    Field { name: "age".to_string(), type_: "Integer".to_string(), optional: false },
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
                    Field { name: "request".to_string(), type_: "Ping".to_string(), optional: false },
                    Field { name: "response".to_string(), type_: "Pong".to_string(), optional: false },
                ],
            }))
        )
    }
}
