use crate::schema;

struct Generator {
    level: usize,
    output: String,
}

impl Generator {
    fn new() -> Self {
        Self {
            level: 0,
            output: String::new(),
        }
    }
    fn begin(&mut self, line: &str) {
        self.line(line);
        self.level += 1;
    }
    fn end(&mut self, line: &str) {
        self.level -= 1;
        self.line(line);
    }
    fn line(&mut self, line: &str) {
        for _ in 0..self.level {
            self.output += "    ";
        }
        self.output += line;
        self.output += "\n";
    }
}

impl Into<String> for Generator {
    fn into(self) -> String {
        self.output
    }
}

pub fn gen(doc: &schema::Document) -> String {
    let mut gen = Generator::new();
    gen.line("// GENERATED CODE - DO NOT EDIT!");
    gen.line("");
    // XXX those types should be moved into a webwire npm package
    gen.line("export type UUID = string;");
    gen.line("export type Result<T, E> = { Ok: T } | { Error: E }");
    gen.line("");
    gen_namespace(&doc.ns, &mut gen);
    gen.into()
}

fn gen_namespace(ns: &schema::Namespace, gen: &mut Generator) {
    for type_ in ns.types.values() {
        gen_type(&*type_.borrow(), gen);
    }
    for service in ns.services.values() {
        gen_service(service, gen);
    }
    for child_ns in ns.namespaces.values() {
        gen.begin(&format!("namespace {} {{", child_ns.name()));
        gen_namespace(child_ns, gen);
        gen.end("}");
    }
}

fn gen_type(type_: &schema::UserDefinedType, gen: &mut Generator) {
    match type_ {
        schema::UserDefinedType::Enum(enum_) => gen_enum(enum_, gen),
        schema::UserDefinedType::Struct(struct_) => gen_struct(struct_, gen),
        schema::UserDefinedType::Fieldset(fieldset) => gen_fieldset(fieldset, gen),
    }
}

fn gen_enum(enum_: &schema::Enum, gen: &mut Generator) {
    gen.begin(&format!("export interface {} {{", enum_.fqtn.name));
    for variant in &enum_.variants {
        let value_type = match &variant.value_type {
            Some(value_type) => gen_typeref(value_type),
            None => "null".to_string(),
        };
        // FIXME this is not the way enum variants should be generated. Actually
        // a pattern matching where one value is required would be better.
        gen.line(&format!("{}?: {},", variant.name, value_type));
    }
    gen.end("}");
}

fn gen_struct(struct_: &schema::Struct, gen: &mut Generator) {
    let generics = if struct_.generics.is_empty() {
        "".to_string()
    } else {
        format!("<{}>", struct_.generics.join(", "))
    };
    gen.begin(&format!(
        "export interface {}{} {{",
        struct_.fqtn.name, generics
    ));
    for field in struct_.fields.iter() {
        let opt = if field.required { "" } else { "?" };
        gen.line(&format!(
            "{}{}: {},",
            field.name,
            opt,
            gen_typeref(&field.type_)
        ));
    }
    gen.end("}");
}

fn gen_fieldset(fieldset: &schema::Fieldset, gen: &mut Generator) {
    let generics = if fieldset.generics.is_empty() {
        "".to_string()
    } else {
        format!("<{}>", fieldset.generics.join(", "))
    };
    gen.begin(&format!(
        "export interface {}{} {{",
        fieldset.fqtn.name, generics
    ));
    for field in fieldset.fields.iter() {
        // FIXME add support for optional fields
        let opt = if field.optional { "?" } else { "" };
        gen.line(&format!(
            "{}{}: {},",
            field.name,
            opt,
            gen_typeref(&field.field.as_ref().unwrap().type_)
        ));
    }
    gen.end("}");
}

fn gen_service(service: &schema::Service, gen: &mut Generator) {
    gen.begin(&format!("export interface {} {{", service.name));
    for method in service.methods.iter() {
        let input = match &method.input {
            Some(t) => format!("input: {}", gen_typeref(&t)),
            None => String::new(),
        };
        let output = match &method.output {
            Some(t) => gen_typeref(t),
            None => "void".to_string(),
        };
        gen.line(&format!("{}({}): Promise<{}>,", method.name, input, output));
    }
    gen.end("}");
}

pub fn gen_typeref(type_: &schema::Type) -> String {
    match type_ {
        schema::Type::Boolean => "boolean".to_string(),
        schema::Type::Integer => "number".to_string(),
        schema::Type::Float => "number".to_string(),
        schema::Type::String => "string".to_string(),
        schema::Type::UUID => "UUID".to_string(),
        schema::Type::Date => "Date".to_string(),
        schema::Type::Time => "Time".to_string(),
        schema::Type::DateTime => "DateTime".to_string(),
        // complex types
        schema::Type::Array(array) => format!("Array<{}>", gen_typeref(&array.item_type)),
        schema::Type::Map(map) => format!(
            "Map<{}, {}>",
            gen_typeref(&map.key_type),
            gen_typeref(&map.value_type)
        ),
        // named
        schema::Type::Ref(typeref) => {
            if !typeref.generics.is_empty() {
                let generics = typeref
                    .generics
                    .iter()
                    .map(gen_typeref)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}<{}>", typeref.fqtn.name, generics)
            } else {
                typeref.fqtn.name.to_string()
            }
        }
    }
}
