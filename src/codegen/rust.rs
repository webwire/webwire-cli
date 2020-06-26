use ::codegen::Scope;
use ::codegen;

use crate::schema;

pub fn gen(doc: &schema::Document) -> String {
    let mut scope = Scope::new();
    convert_ns(&doc.ns, &mut scope);
    scope.to_string()
}

fn convert_ns(ns: &schema::Namespace, scope: &mut codegen::Scope) {
    for type_ in ns.types.values() {
        match &*type_.borrow() {
            schema::UserDefinedType::Enum(enum_) => {
                let code_enum = scope.new_enum(&enum_.fqtn.name);
                code_enum.vis("pub");
                for variant in enum_.variants.iter() {
                    let code_variant = code_enum.new_variant(&variant.name);
                    if let Some(value_type) = &variant.value_type {
                        code_variant.tuple(&s(&convert_type(&value_type)));
                    }
                }
            }
            schema::UserDefinedType::Struct(struct_) => {
                let code_struct = scope.new_struct(&struct_.fqtn.name);
                code_struct.vis("pub");
                for generic in struct_.generics.iter() {
                    code_struct.generic(&generic);
                }
                for field in struct_.fields.iter() {
                    // FIXME add support for optional fields
                    let code_field = code_struct.field(&field.name, convert_type(&field.type_));
                    // FIXME change visibilty to pub
                }
            }
            schema::UserDefinedType::Fieldset(fieldset_) => {
                // FIXME
            }
        }
    }
    for ns in ns.namespaces.values() {
        let ns_name = ns.path.last().unwrap();
        let module = scope.new_module(ns_name);
        convert_ns(&ns, module.scope());
    }
}

fn s(ty: &codegen::Type) -> String {
    let mut s = String::new();
    let mut fmt = codegen::Formatter::new(&mut s);
    ty.fmt(&mut fmt).unwrap();
    s
}

fn convert_type(type_: &schema::Type) -> codegen::Type {
    match type_ {
        schema::Type::Boolean => codegen::Type::new("bool"),
        schema::Type::Integer => codegen::Type::new("i64"),
        schema::Type::Float => codegen::Type::new("f64"),
        schema::Type::String => codegen::Type::new("String"),
        schema::Type::UUID => codegen::Type::new("UUID"),
        schema::Type::Date => codegen::Type::new("Date"),
        schema::Type::Time => codegen::Type::new("Time"),
        schema::Type::DateTime => codegen::Type::new("DateTime"),
        // complex types
        schema::Type::Array(array) => {
            let mut code_type = codegen::Type::new("Vec");
            code_type.generic(convert_type(&array.item_type));
            code_type
        }
        schema::Type::Map(map) => {
            let mut code_type = codegen::Type::new("HashMap");
            code_type.generic(convert_type(&map.key_type));
            code_type.generic(convert_type(&map.value_type));
            code_type
        }
        // named
        schema::Type::Ref(typeref) => {
            let mut code_type = codegen::Type::new(&convert_fqtn(&typeref.fqtn));
            for generic in typeref.generics.iter() {
                code_type.generic(convert_type(generic));
            }
            code_type
        }
    }
}

fn convert_fqtn(fqtn: &schema::FQTN) -> String {
    if fqtn.ns.is_empty() {
        fqtn.name.clone()
    } else {
        format!("::{}::{}", fqtn.ns.join("::"), fqtn.name)
    }
}
