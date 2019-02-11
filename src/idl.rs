use nom::types::CompleteStr;

#[derive(Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_: String,
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub name: String,
    pub fields: Vec<Field>,
}

named!(parse_identifier<CompleteStr, String>, do_parse!(
    first: take_while1!(char::is_alphabetic) >>
    rest: take_while!(char::is_alphanumeric) >>
    (first.0.to_owned() + rest.0)
));

named!(parse_field<CompleteStr, Field>, do_parse!(
    name: parse_identifier >>
    take_while!(char::is_whitespace) >>
    char!(':') >>
    take_while!(char::is_whitespace) >>
    type_: parse_identifier >>
    (Field { name: name.to_string(), type_: type_.to_string() })
));

named!(parse_field_separator<CompleteStr, ()>, do_parse!(
    take_while!(char::is_whitespace) >>
    char!(',') >>
    take_while!(char::is_whitespace) >>
    ()
));

named!(parse_service<CompleteStr, Service>, do_parse!(
        tag!("service") >>
        take_while1!(char::is_whitespace) >>
        name: parse_identifier >>
        // FIXME parse fields
        take_while!(char::is_whitespace) >>
        char!('{') >>
        take_while!(char::is_whitespace) >>
        fields: separated_list!(parse_field_separator, parse_field) >>
        take_while!(char::is_whitespace) >>
        opt!(char!(',')) >> // trailing comma
        take_while!(char::is_whitespace) >>
        char!('}') >>
        (Service { name: name.to_string(), fields: fields })
));

#[test]
fn test_parse_identifier() {
    assert_eq!(
        parse_identifier(CompleteStr("test")),
        Ok((CompleteStr(""), "test".to_string()))
    );
}


#[test]
fn test_parse_service() {
    let contents = [
        "service Pinger{}",
        "service Pinger {}",
        "service Pinger{ }",
        "service Pinger { }",
    ];
    for content in contents.iter() {
        assert_eq!(
            parse_service(CompleteStr(content)),
            Ok((CompleteStr(""), Service {
                name: CompleteStr("Pinger").to_string(),
                fields: vec![],
            }))
        );
    }
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
            parse_field(CompleteStr(content)),
            Ok((CompleteStr(""), Field {
                name: "foo".to_string(),
                type_: "FooType".to_string(),
            }))
        );
    }
}

#[test]
fn test_parse_service_with_fields() {
    let contents = [
        // no whitespace
        "service Pinger {request: Ping,response: Pong}",
        // trailing whitespace after comma
        "service Pinger {request: Ping, response: Pong}",
        // leading whitespace before comma
        "service Pinger {request: Ping ,response: Pong}",
        // whitespace between braces
        "service Pinger { request: Ping,response: Pong }",
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
            parse_service(CompleteStr(content)),
            Ok((CompleteStr(""), Service {
                name: "Pinger".to_string(),
                fields: vec![
                    Field { name: "request".to_string(), type_: "Ping".to_string() },
                    Field { name: "response".to_string(), type_: "Pong".to_string() },
                ],
            }))
        )
    }
}
