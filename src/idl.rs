use std::collections::BTreeMap;

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

named!(parse_field<&str, Field>, do_parse!(
    name: take_while1!(char::is_alphanumeric) >>
    take_while!(char::is_whitespace) >>
    tag!(":") >>
    take_while!(char::is_whitespace) >>
    type_: take_while1!(char::is_alphanumeric) >>
    (Field { name: name.to_string(), type_: type_.to_string() })
));

named!(parse_field_separator<&str, ()>, do_parse!(
    take_while!(char::is_whitespace) >>
    tag!(",") >>
    take_while!(char::is_whitespace) >>
    ()
));

named!(parse_service<&str, Service>, do_parse!(
        tag!("service") >>
        take_while1!(char::is_whitespace) >>
        name: take_while1!(char::is_alphanumeric) >>
        // FIXME parse fields
        take_while!(char::is_whitespace) >>
        char!('{') >>
        take_while!(char::is_whitespace) >>
        fields: separated_list!(parse_field_separator, parse_field) >>
        take_while!(char::is_whitespace) >>
        opt!(tag!(",")) >> // trailing comma
        take_while!(char::is_whitespace) >>
        char!('}') >>
        (Service { name: name.to_string(), fields: fields })
));

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
            parse_service(content),
            Ok(("", Service {
                name: "Pinger".to_string(),
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
            parse_field(content),
            Ok(("", Field {
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
    let mut fields = BTreeMap::<String, String>::new();
    fields.insert("request".into(), "Ping".into());
    fields.insert("response".into(), "Pong".into());
    for content in contents.iter() {
        assert_eq!(
            parse_service(content),
            Ok(("", Service {
                name: "Pinger".to_string(),
                fields: vec![
                    Field { name: "request".to_string(), type_: "Ping".to_string() },
                    Field { name: "response".to_string(), type_: "Pong".to_string() },
                ],
            }))
        )
    }
}
