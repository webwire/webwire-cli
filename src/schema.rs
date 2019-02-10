extern crate serde_yaml;

use std::collections::BTreeMap as Map;
use serde_yaml::Error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StringField {
    max_length: Option<u32>
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ArrayField {
    item: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MapField {
    key: String,
    value: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(non_camel_case_types)]
pub enum FieldDetails {
    string(StringField),
    array(ArrayField),
    map(MapField),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Field {
    Name(String),
    Details(FieldDetails)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldsetItem {
    optional: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Type {
    fields: Map<String, Field>,
    fieldsets: Option<Map<String, Map<String, FieldsetItem>>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Endpoint {
    request: String,
    response: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Service {
    endpoints: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    types: Map<String, Type>,
    endpoints: Map<String, Endpoint>,
    services: Map<String, Service>,
}

pub fn parse_string(s: &String) -> Result<Schema, Error> {
    serde_yaml::from_str(s)
}
