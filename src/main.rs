use std::fs::read;

use webwire::codegen::rust::gen;
use webwire::idl;
use webwire::schema;

fn main() {
    let content = read("tests/idl_complete.ww").unwrap();
    let s = String::from_utf8(content).unwrap();
    let idl = idl::parse_document(&s).unwrap();
    let doc = schema::Document::from_idl(&idl).unwrap();
    println!("{}", gen(&doc));
}
