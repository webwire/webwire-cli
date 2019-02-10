const EXAMPLE_SCHEMA: &str = "
{
    types: {
        UserRequest: {
            fields: {
                email: string,
            }
        },
        Name: {
            fields: {

            },
            fieldsets: {
                info: {
                    first_name: {},
                    last_name: {}
                }
            }
        },
        User: {
            fields: {
                id: uuid,
                email: email,
                password: { type: string, max_length: 64 },
                is_admin: bool,
                # name: { type: Name, fieldset: info },
                full_name: string,
            },
            fieldsets: {
                read: {
                    id: {},
                    email: {},
                    is_admin: {},
                    full_name: {}
                },
                write: {
                    id: {},
                    email: { required: false },
                    password: { required: false },
                    is_admin: { required: false },
                }
            }
        },
        UserList: {
            fields: {
                count: int,
                users: { type: array, item: User },
                permissions: { type: map, key: string, value: string },
            }
        }
    },
    endpoints: {
        get_user: {
            request: UserRequest,
            response: User,
        }
    },
    services: {
        server: {
            endpoints: [
                get_user
            ]
        }
    }
}
";

#[test]
fn test_schema_loader() {
    let result = ninjapi::schema::parse_string(&EXAMPLE_SCHEMA.to_string());
    match result {
        Ok(_schema) => (),
        Err(error) => assert!(false, format!("{}", error)),
    }
}

