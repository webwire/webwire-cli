use proc_macro2::TokenStream;
use quote::quote;

use crate::schema;

pub fn gen(doc: &schema::Document) -> String {
    let stream = generate(doc);
    let code = format!("{}", stream);
    code
}

fn optional(stream: TokenStream) -> TokenStream {
    quote! {
        Option<#stream>
    }
}

pub fn generate(doc: &schema::Document) -> TokenStream {
    let namespace = gen_namespace(&doc.ns);
    quote! {
        #[allow(dead_code)]
        #namespace
    }
}

fn gen_namespace(ns: &schema::Namespace) -> TokenStream {
    let mut stream = TokenStream::new();
    for type_ in ns.types.values() {
        let type_stream = gen_type(&*type_.borrow());
        stream.extend(type_stream);
    }
    for service in ns.services.values() {
        let service_stream = gen_service(service);
        stream.extend(service_stream);
        let provider_stream = gen_provider(service);
        stream.extend(provider_stream);
        let consumer_stream = gen_consumer(service);
        stream.extend(consumer_stream);
    }
    for child_ns in ns.namespaces.values() {
        let child_ns_name = quote::format_ident!("{}", child_ns.name());
        let child_ns_stream = gen_namespace(child_ns);
        stream.extend(quote! {
            pub mod #child_ns_name {
                #child_ns_stream
            }
        });
    }
    stream
}

fn gen_type(type_: &schema::UserDefinedType) -> TokenStream {
    match type_ {
        schema::UserDefinedType::Enum(enum_) => gen_enum(enum_),
        schema::UserDefinedType::Struct(struct_) => gen_struct(struct_),
        schema::UserDefinedType::Fieldset(fieldset) => gen_fieldset(fieldset),
    }
}

fn gen_enum(enum_: &schema::Enum) -> TokenStream {
    let name = quote::format_ident!("{}", &enum_.fqtn.name);
    let variants = gen_enum_variants(enum_);
    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub enum #name {
            #variants
        }
    }
}

fn gen_enum_variants(enum_: &schema::Enum) -> TokenStream {
    let mut stream = TokenStream::new();
    for variant in enum_.variants.iter() {
        stream.extend(gen_enum_variant(variant));
    }
    stream
}

fn gen_enum_variant(variant: &schema::EnumVariant) -> TokenStream {
    let name = quote::format_ident!("{}", variant.name);
    if let Some(value_type) = &variant.value_type {
        let value_type = gen_typeref(value_type);
        quote! {
            #name(#value_type),
        }
    } else {
        quote! {
            #name,
        }
    }
}

fn gen_struct(struct_: &schema::Struct) -> TokenStream {
    let name = quote::format_ident!("{}", &struct_.fqtn.name);
    let fields = gen_struct_fields(struct_);
    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #name {
            #fields
        }
    }
}

fn gen_struct_fields(struct_: &schema::Struct) -> TokenStream {
    let mut stream = TokenStream::new();
    for field in struct_.fields.iter() {
        stream.extend(gen_struct_field(field))
    }
    stream
}

fn gen_struct_field(field: &schema::Field) -> TokenStream {
    let name = quote::format_ident!("{}", field.name);
    let mut type_ = gen_typeref(&field.type_);
    if field.optional {
        type_ = optional(type_);
    }
    quote! {
        pub #name: #type_,
    }
}

fn gen_fieldset(fieldset: &schema::Fieldset) -> TokenStream {
    let name = quote::format_ident!("{}", &fieldset.fqtn.name);
    let fields = gen_fieldset_fields(fieldset);
    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub struct #name {
            #fields
        }
    }
}

fn gen_fieldset_fields(struct_: &schema::Fieldset) -> TokenStream {
    let mut stream = TokenStream::new();
    for field in struct_.fields.iter() {
        stream.extend(gen_fieldset_field(field))
    }
    stream
}

fn gen_fieldset_field(field: &schema::FieldsetField) -> TokenStream {
    let name = quote::format_ident!("{}", field.name);
    let mut type_ = gen_typeref(&field.field.as_ref().unwrap().type_);
    if field.optional {
        type_ = optional(type_);
    }
    quote! {
        pub #name: #type_,
    }
}

fn gen_service(service: &schema::Service) -> TokenStream {
    let service_name = quote::format_ident!("{}", &service.name);
    let methods = gen_service_methods(&service);
    quote! {
        #[::async_trait::async_trait]
        pub trait #service_name<S: ::std::marker::Sync + ::std::marker::Send> {
            #methods
        }
    }
}

fn gen_service_methods(service: &schema::Service) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods.iter() {
        let signature = gen_service_method_signature(method);
        stream.extend(quote! {
            #signature;
        })
    }
    stream
}

fn gen_service_method_signature(method: &schema::Method) -> TokenStream {
    let name = quote::format_ident!("{}", method.name);
    let input = match &method.input {
        Some(type_) => gen_typeref(type_),
        None => quote! { () },
    };
    let output = match &method.output {
        Some(type_) => gen_typeref(type_),
        None => quote! { () },
    };
    quote! {
        async fn #name(&self, input: &#input) -> Result<#output, ::webwire::ProviderError>
    }
}

fn gen_provider(service: &schema::Service) -> TokenStream {
    let service_name = quote::format_ident!("{}", service.name);
    let service_name_str = &service.name;
    let provider_name = quote::format_ident!("{}Provider", service.name);
    let matches = gen_provider_matches(&service);
    quote! {
        pub struct #provider_name<F>(pub F);
        // NamedProvider impl
        impl<F: Sync + Send, S: Sync + Send, T: Sync + Send> ::webwire::NamedProvider<S> for #provider_name<F>
        where
            F: Fn(::std::sync::Arc<S>) -> T,
            T: #service_name<S> + 'static,
        {
            const NAME: &'static str = #service_name_str;
        }
        // Provider impl
        impl<F: Sync + Send, S: Sync + Send, T: Sync + Send> ::webwire::Provider<S> for #provider_name<F>
        where
            F: Fn(::std::sync::Arc<S>) -> T,
            T: #service_name<S> + 'static,
        {
            fn call(
                &self,
                session: &::std::sync::Arc<S>,
                method: &str,
                input: ::bytes::Bytes,
            ) -> ::futures::future::BoxFuture<'static, Result<::bytes::Bytes, ::webwire::ProviderError>> {
                let service = self.0(session.clone());
                match method {
                    #matches
                    _ => Box::pin(::futures::future::ready(Err(::webwire::ProviderError::MethodNotFound))),
                }
            }
        }
    }
}

fn gen_provider_matches(service: &schema::Service) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods.iter() {
        let name = quote::format_ident!("{}", method.name);
        let name_str = &method.name;
        let input = match &method.input {
            Some(type_) => gen_typeref(type_),
            None => quote! { () },
        };
        /*
        let output = match &method.output {
            Some(type_) => gen_typeref(type_),
            None => quote! { () },
        };
        */
        let deserialize_request = if method.input.is_none() {
            quote! { ::bytes::Bytes::new() }
        } else {
            quote! {
                serde_json::from_slice::<#input>(&input)
                    .map_err(|e| ::webwire::ProviderError::DeserializerError(e))?;
            }
        };
        stream.extend(quote! {
            #name_str => Box::pin(async move {
                let input = #deserialize_request;
                let output = service.#name(&input).await?;
                let response = serde_json::to_vec(&output)
                    .map_err(|e| ::webwire::ProviderError::SerializerError(e))
                    .map(::bytes::Bytes::from)?;
                Ok(response)
            }),
        });
    }
    stream
}

fn gen_consumer(service: &schema::Service) -> TokenStream {
    let service_name = quote::format_ident!("{}", service.name);
    let service_name_str = &service.name;
    let consumer_name = quote::format_ident!("{}Consumer", service.name);
    let consumer_methods = gen_consumer_methods(&service);
    quote! {
        pub struct #consumer_name<'a>(pub &'a (dyn ::webwire::Consumer + ::std::marker::Sync + ::std::marker::Send));
        /*
        #[::async_trait::async_trait]
        impl<'a> ::webwire::Consumer for #consumer_name<'a> {
            fn name(&self) -> &'static str {
                #service_name_str
            }
        }
        */
        impl<'a> #consumer_name<'a> {
            #consumer_methods
        }
    }
}

fn gen_consumer_methods(service: &schema::Service) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods.iter() {
        let signature = gen_consumer_method_signature(method);
        let method_name_str = &method.name;
        stream.extend(quote! {
            #signature {
                let data = serde_json::to_vec(input)
                    .map_err(|e| ::webwire::ConsumerError::SerializerError(e))?;
                let output = self.0.call(#method_name_str, data.into()).await?;
                let response = serde_json::from_slice(&output)
                    .map_err(|e| ::webwire::ConsumerError::DeserializerError(e))?;
                Ok(response)
            }
        })
    }
    stream
}

fn gen_consumer_method_signature(method: &schema::Method) -> TokenStream {
    let name = quote::format_ident!("{}", method.name);
    let input = match &method.input {
        Some(type_) => gen_typeref(type_),
        None => quote! { () },
    };
    let output = match &method.output {
        Some(type_) => gen_typeref(type_),
        None => quote! { () },
    };
    quote! {
        pub async fn #name(&self, input: &#input) -> Result<#output, ::webwire::ConsumerError>
    }
}

fn gen_typeref(type_: &schema::Type) -> TokenStream {
    match type_ {
        schema::Type::Boolean => quote! { bool },
        schema::Type::Integer => quote! { i64 },
        schema::Type::Float => quote! { f64 },
        schema::Type::String => quote! { String },
        schema::Type::UUID => quote! { ::uuid::Uuid },
        schema::Type::Date => quote! { Date },
        schema::Type::Time => quote! { Time },
        schema::Type::DateTime => quote! { DateTime },
        // complex types
        schema::Type::Array(array) => {
            let item_type = gen_typeref(&array.item_type);
            quote! {
                std::vec::Vec<#item_type>
            }
        }
        schema::Type::Map(map) => {
            let key_type = gen_typeref(&map.key_type);
            let value_type = gen_typeref(&map.value_type);
            quote! {
                std::collections::HashMap<#key_type, #value_type>
            }
        }
        // named
        schema::Type::Ref(typeref) => {
            let mut generics_stream = TokenStream::new();
            if !typeref.generics.is_empty() {
                for generic in typeref.generics.iter() {
                    let type_ = gen_typeref(generic);
                    generics_stream.extend(quote! {
                        #type_,
                    })
                }
                generics_stream = quote! {
                    < #generics_stream >
                }
            }
            // FIXME fqtn
            match &*typeref.fqtn.name {
                // FIXME `None` should be made into a buitlin type
                "None" => quote! { () },
                name => {
                    let name = quote::format_ident!("{}", name);
                    quote! {
                        #name #generics_stream
                    }
                }
            }
        }
    }
}
