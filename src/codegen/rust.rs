use quote::quote;
use proc_macro2::TokenStream;

use crate::schema;

pub fn gen(doc: &schema::Document) -> String {
    let stream = generate(doc);
    let code = format!("{}", stream);
    code
}

pub fn generate(doc: &schema::Document) -> TokenStream {
    let namespace = generate_namespace(&doc.ns);
    quote! {
        use async_trait::async_trait;

        use webwire;

        #namespace
    }
}

pub fn generate_namespace(ns: &schema::Namespace) -> TokenStream {
    let mut stream = TokenStream::new();
    for type_ in ns.types.values() {
        let type_stream = generate_type(&*type_.borrow());
        stream.extend(type_stream);
    }
    for service in ns.services.values() {
        let service_stream = generate_service(service);
        stream.extend(service_stream);
    }
    for child_ns in ns.namespaces.values() {
        let child_ns_name = quote::format_ident!("{}", child_ns.name());
        let child_ns_stream = generate_namespace(child_ns);
        stream.extend(quote! {
            mod #child_ns_name {
                #child_ns_stream
            }
        });
    }
    stream
}

pub fn generate_type(type_: &schema::UserDefinedType) -> TokenStream {
    match type_ {
        schema::UserDefinedType::Enum(enum_) => {
            generate_enum(enum_)
        }
        schema::UserDefinedType::Struct(struct_) => {
            generate_struct(struct_)
        }
        schema::UserDefinedType::Fieldset(fieldset) => {
            generate_fieldset(fieldset)
        }
    }
}

pub fn generate_enum(enum_: &schema::Enum) -> TokenStream {
    let name = quote::format_ident!("{}", &enum_.fqtn.name);
    let variants = generate_enum_variants(enum_);
    quote! {
        enum #name {
            #variants
        }
    }
}

pub fn generate_enum_variants(enum_: &schema::Enum) -> TokenStream {
    let mut stream = TokenStream::new();
    for variant in enum_.variants.iter() {
        stream.extend(generate_enum_variant(variant));
    }
    stream
}

pub fn generate_enum_variant(variant: &schema::EnumVariant) -> TokenStream {
    let name = quote::format_ident!("{}", variant.name);
    if let Some(value_type) = &variant.value_type {
        let value_type = generate_typeref(value_type);
        quote! {
            #name(#value_type),
        }
    } else {
        quote! {
            #name,
        }
    }
}

pub fn generate_struct(struct_: &schema::Struct) -> TokenStream {
    let name = quote::format_ident!("{}", &struct_.fqtn.name);
    let fields = generate_struct_fields(struct_);
    quote! {
        pub struct #name {
            #fields
        }
    }
}

pub fn generate_struct_fields(struct_: &schema::Struct) -> TokenStream {
    let mut stream = TokenStream::new();
    for field in struct_.fields.iter() {
        stream.extend(generate_struct_field(field))
    }
    stream
}

pub fn generate_struct_field(field: &schema::Field) -> TokenStream {
    let name = quote::format_ident!("{}", field.name);
    let type_ = generate_typeref(&field.type_);
    // FIXME add support for required modifier
    quote! {
        pub #name: #type_,
    }
}

pub fn generate_fieldset(fieldset: &schema::Fieldset) -> TokenStream {
    // FIXME implement
    quote! {
    }
}

pub fn generate_service(service: &schema::Service) -> TokenStream {
    let name = quote::format_ident!("{}", &service.name);
    let methods = generate_service_methods(&service);
    quote! {
        #[async_trait]
        pub trait #name {
            #methods
        }
    }
}

pub fn generate_service_methods(service: &schema::Service) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods.iter() {
        let name = quote::format_ident!("{}", method.name);
        let input = match &method.input {
            Some(type_) => generate_typeref(type_),
            None => quote! {}
        };
        let output = match &method.output {
            Some(type_) => generate_typeref(type_),
            None => quote! { () }
        };
        stream.extend(quote! {
            async fn #name(&self, request: &webwire::Request<#input>) -> webwire::Response<#output>;
        })
    }
    stream
}

pub fn generate_typeref(type_: &schema::Type) -> TokenStream {
    match type_ {
        schema::Type::Boolean => quote! { bool },
        schema::Type::Integer => quote! { i64 },
        schema::Type::Float => quote! { f64 },
        schema::Type::String => quote! { String },
        schema::Type::UUID => quote! { UUID },
        schema::Type::Date => quote! { Date },
        schema::Type::Time => quote! { Time },
        schema::Type::DateTime => quote! { DateTime },
        // complex types
        schema::Type::Array(array) => {
            let item_type = generate_typeref(&array.item_type);
            quote! {
                Vec<#item_type>
            }
        }
        schema::Type::Map(map) => {
            let key_type = generate_typeref(&map.key_type);
            let value_type = generate_typeref(&map.value_type);
            quote! {
                Map<#key_type, #value_type>
            }
        }
        // named
        schema::Type::Ref(typeref) => {
            let mut generics_stream = TokenStream::new();
            if !typeref.generics.is_empty() {
                for generic in typeref.generics.iter() {
                    let type_ = generate_typeref(generic);
                    generics_stream.extend(quote! {
                        #type_,
                    })
                }
                generics_stream = quote! {
                    < #generics_stream >
                }
            }
            // FIXME fqtn
            let name = quote::format_ident!("{}", &typeref.fqtn.name);
            quote! {
                #name #generics_stream
            }
        }
    }
}
