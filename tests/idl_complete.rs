const EXAMPLE_SCHEMA: &str = include_str!("./idl_complete.ninjapi");

#[test]
fn test_schema_loader() {
    use ninjapi::idl::*;
    let result = parse_document(EXAMPLE_SCHEMA);
    assert_eq!(
        Ok(Document { parts: vec![
            DocumentPart::Enum(Enum { name: "UserState".to_string(), values: vec![
                "ACTIVE".to_string(),
                "INACTIVE".to_string(),
                "BANNED".to_string()
            ]}),
            DocumentPart::Struct(Struct { name: "UserRequest".to_string(), fields: vec![
                Field { name: "email".to_string(), type_: Type::Named("Email".to_string()), optional: false, options: vec![] }
            ]}),
            DocumentPart::Struct(Struct { name: "Name".to_string(), fields: vec![
                Field { name: "prefix".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(0), Some(50)) }
                ]},
                Field { name: "first_name".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(0), Some(100)) }
                ]},
                Field { name: "middle_name".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(0), Some(100)) }
                ]},
                Field { name: "last_name".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(0), Some(100)) },
                    FieldOption { name: "help".to_string(), value: Value::String("aka. family name".to_string()) }
                ]},
                Field { name: "suffix".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(0), Some(50)) }] },
                Field { name: "full_name".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![] }]
            }),
            DocumentPart::Fieldset(Fieldset { name: "NameUpdate".to_string(), struct_name: "Name".to_string(), fields: vec![
                FieldsetField { name: "prefix".to_string(), optional: true },
                FieldsetField { name: "first_name".to_string(), optional: true },
                FieldsetField { name: "middle_name".to_string(), optional: true },
                FieldsetField { name: "last_name".to_string(), optional: true },
                FieldsetField { name: "suffix".to_string(), optional: true }]
            }),
            DocumentPart::Struct(Struct { name: "User".to_string(), fields: vec![
                Field { name: "id".to_string(), type_: Type::Named("UUID".to_string()), optional: false, options: vec![] },
                Field { name: "email".to_string(), type_: Type::Named("Email".to_string()), optional: false, options: vec![] },
                Field { name: "name".to_string(), type_: Type::Named("Name".to_string()), optional: false, options: vec![] },
                Field { name: "password".to_string(), type_: Type::Named("String".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(5), Some(64)) }
                ]},
                Field { name: "is_admin".to_string(), type_: Type::Named("Boolean".to_string()), optional: false, options: vec![] }]
            }),
            DocumentPart::Fieldset(Fieldset { name: "UserRead".to_string(), struct_name: "User".to_string(), fields: vec![
                FieldsetField { name: "id".to_string(), optional: false },
                FieldsetField { name: "email".to_string(), optional: false },
                FieldsetField { name: "is_admin".to_string(), optional: false },
                FieldsetField { name: "full_name".to_string(), optional: false }]
            }),
            DocumentPart::Fieldset(Fieldset { name: "UserWrite".to_string(), struct_name: "User".to_string(), fields: vec![
                FieldsetField { name: "id".to_string(), optional: false },
                FieldsetField { name: "email".to_string(), optional: true },
                FieldsetField { name: "is_admin".to_string(), optional: true },
                FieldsetField { name: "name".to_string(), optional: true },
                FieldsetField { name: "password".to_string(), optional: true }]
            }),
            DocumentPart::Struct(Struct { name: "UserListRequest".to_string(), fields: vec![
                Field { name: "offset".to_string(), type_: Type::Named("Integer".to_string()), optional: true, options: vec![
                    FieldOption { name: "size".to_string(), value: Value::Integer(32) },
                    FieldOption { name: "range".to_string(), value: Value::Range(Some(0), None) }
                ]},
                Field { name: "limit".to_string(), type_: Type::Named("Integer".to_string()), optional: true, options: vec![
                    FieldOption { name: "range".to_string(), value: Value::Range(Some(1), Some(200)) }] }]
            }),
            DocumentPart::Struct(Struct { name: "UserList".to_string(), fields: vec![
                Field { name: "count".to_string(), type_: Type::Named("Integer".to_string()), optional: false, options: vec![
                    FieldOption { name: "range".to_string(), value: Value::Range(Some(0), Some(65535)) },
                    FieldOption { name: "help".to_string(), value: Value::String("Count of objects returned".to_string()) }
                ]},
                Field { name: "users".to_string(), type_: Type::Array("User".to_string()), optional: false, options: vec![
                    FieldOption { name: "length".to_string(), value: Value::Range(Some(0), Some(128)) }] },
                Field { name: "permissions".to_string(), type_: Type::Map("UUID".to_string(), "String".to_string()), optional: false, options: vec![] }] }),
            DocumentPart::Enum(Enum { name: "GetError".to_string(), values: vec![
                "PermissionDenied".to_string(),
                "DoesNotExist".to_string()
            ]}),
            DocumentPart::Enum(Enum { name: "ListError".to_string(), values: vec![
                "PermissionDenied".to_string()
            ]}),
            DocumentPart::Endpoint(Endpoint {
                name: "get_version".to_string(),
                request: None,
                response: Some("String".to_string()),
                error: None
            }),
            DocumentPart::Endpoint(Endpoint {
                name: "user_get".to_string(),
                request: Some("UserRequest".to_string()),
                response: Some("User".to_string()),
                error: Some("GetError".to_string())
            }),
            DocumentPart::Endpoint(Endpoint {
                name: "user_list".to_string(),
                request: Some("UserListRequest".to_string()),
                response: Some("UserList".to_string()),
                error: Some("ListError".to_string())
            }),
            DocumentPart::Service(Service { name: "server".to_string(), operations: vec![
                "get_version".to_string(),
                "user_get".to_string(),
                "user_list".to_string()
            ]}),
            DocumentPart::Service(Service { name: "client".to_string(), operations: vec![
                "get_version".to_string()
            ]})
        ]}),
        result
    );
}
