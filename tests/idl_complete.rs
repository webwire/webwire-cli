const EXAMPLE_SCHEMA: &str = include_str!("./idl_complete.ninjapi");

#[test]
fn test_schema_loader() {
    let result = ninjapi::idl::parse_document(EXAMPLE_SCHEMA);
    assert_eq!(Ok(("", ninjapi::idl::Document { parts: vec![] })), result);
}
