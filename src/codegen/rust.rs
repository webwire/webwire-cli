use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{self, TypeRef};

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
        let type_stream = gen_type(type_, &ns.path);
        stream.extend(type_stream);
    }
    for service in ns.services.values() {
        let service_stream = gen_service(service, &ns.path);
        stream.extend(service_stream);
        let provider_stream = gen_provider(service, &ns.path);
        stream.extend(provider_stream);
        let consumer_stream = gen_consumer(service, &ns.path);
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

fn gen_type(type_: &schema::UserDefinedType, ns: &[String]) -> TokenStream {
    match type_ {
        schema::UserDefinedType::Enum(enum_) => gen_enum(&enum_.borrow(), ns),
        schema::UserDefinedType::Struct(struct_) => gen_struct(&struct_.borrow(), ns),
        schema::UserDefinedType::Fieldset(fieldset) => gen_fieldset(&fieldset.borrow(), ns),
    }
}

fn gen_enum(enum_: &schema::Enum, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", &enum_.fqtn.name);
    let variants = gen_enum_variants(enum_, ns);
    let mut stream = TokenStream::new();
    stream.extend(quote! {
        #[derive(Clone, Debug, Eq, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
        pub enum #name {
            #variants
        }
    });
    if let Some(extends) = &enum_.extends {
        let extends_typeref = gen_typeref_ref(extends, ns);
        let mut matches = TokenStream::new();
        let extends_enum = enum_.extends_enum().unwrap();
        for variant in extends_enum.borrow().all_variants.iter() {
            let variant_name = quote::format_ident!("{}", variant.name);
            matches.extend(quote! {
                #extends_typeref::#variant_name => #name::#variant_name,
            });
        }
        stream.extend(quote! {
            impl From<#extends_typeref> for #name {
                fn from(other: #extends_typeref) -> Self {
                    match other {
                        #matches
                    }
                }
            }
        });
    }
    stream
}

fn gen_enum_variants(enum_: &schema::Enum, ns: &[String]) -> TokenStream {
    let mut stream = TokenStream::new();
    for variant in enum_.variants.iter() {
        stream.extend(gen_enum_variant(variant, ns));
    }
    if let Some(extends) = enum_.extends_enum() {
        stream.extend(gen_enum_variants(&extends.borrow(), ns));
    }
    stream
}

fn gen_enum_variant(variant: &schema::EnumVariant, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", variant.name);
    if let Some(value_type) = &variant.value_type {
        let value_type = gen_typeref(value_type, ns);
        quote! {
            #name(#value_type),
        }
    } else {
        quote! {
            #name,
        }
    }
}

fn gen_struct(struct_: &schema::Struct, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", &struct_.fqtn.name);
    let fields = gen_struct_fields(struct_, ns);
    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::validator::Validate)]
        pub struct #name {
            #fields
        }
    }
}

fn gen_struct_fields(struct_: &schema::Struct, ns: &[String]) -> TokenStream {
    let mut stream = TokenStream::new();
    for field in struct_.fields.iter() {
        stream.extend(gen_struct_field(field, ns))
    }
    stream
}

fn gen_struct_field(field: &schema::Field, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", field.name);
    let mut type_ = gen_typeref(&field.type_, ns);
    if field.optional {
        type_ = optional(type_);
    }
    let validation_macros = gen_validation_macros(field);
    quote! {
        #validation_macros
        pub #name: #type_,
    }
}

fn gen_validation_macros(field: &schema::Field) -> TokenStream {
    let mut rules = TokenStream::new();
    match field.format.as_deref() {
        Some("email") => rules.extend(quote! { email, }),
        Some("url") => rules.extend(quote! { url, }),
        _ => {}
    }
    match field.length {
        (Some(min), Some(max)) => rules.extend(quote! { length(min=#min, max=#max), }),
        (Some(min), None) => rules.extend(quote! { length(min=#min), }),
        (None, Some(max)) => rules.extend(quote! { length(max=#max), }),
        (None, None) => {}
    }
    if rules.is_empty() {
        quote! {}
    } else {
        quote! {
            #[validate(#rules)]
        }
    }
}

fn gen_fieldset(fieldset: &schema::Fieldset, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", &fieldset.fqtn.name);
    let fields = gen_fieldset_fields(fieldset, ns);
    quote! {
        #[derive(Clone, Debug, Eq, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::validator::Validate)]
        pub struct #name {
            #fields
        }
    }
}

fn gen_fieldset_fields(struct_: &schema::Fieldset, ns: &[String]) -> TokenStream {
    let mut stream = TokenStream::new();
    for field in struct_.fields.iter() {
        stream.extend(gen_fieldset_field(field, ns))
    }
    stream
}

fn gen_fieldset_field(field: &schema::FieldsetField, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", field.name);
    let mut type_ = gen_typeref(&field.field.as_ref().unwrap().type_, ns);
    if field.optional {
        type_ = optional(type_);
    }
    quote! {
        pub #name: #type_,
    }
}

fn gen_service(service: &schema::Service, ns: &[String]) -> TokenStream {
    let service_name = quote::format_ident!("{}", &service.name);
    let methods = gen_service_methods(service, ns);
    quote! {
        #[::async_trait::async_trait]
        pub trait #service_name {
            type Error: Into<::webwire::ProviderError>;
            #methods
        }
    }
}

fn gen_service_methods(service: &schema::Service, ns: &[String]) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods.iter() {
        let signature = gen_service_method_signature(method, ns);
        stream.extend(quote! {
            #signature;
        })
    }
    stream
}

fn gen_service_method_signature(method: &schema::Method, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", method.name);
    let input_arg = match &method.input {
        Some(type_) => {
            let input_type = gen_typeref(type_, ns);
            quote! { input: & #input_type }
        }
        None => quote! {},
    };
    let output = match &method.output {
        Some(type_) => gen_typeref(type_, ns),
        None => quote! { () },
    };
    quote! {
        async fn #name(&self, #input_arg) -> Result<#output, Self::Error>
    }
}

fn gen_provider(service: &schema::Service, ns: &[String]) -> TokenStream {
    let service_name = quote::format_ident!("{}", service.name);
    let service_name_str = if ns.is_empty() {
        service.name.to_owned()
    } else {
        format!("{}.{}", ns.join("."), &service.name)
    };
    let provider_name = quote::format_ident!("{}Provider", service.name);
    let matches = gen_provider_matches(service, ns);
    quote! {
        pub struct #provider_name<F>(pub F);
        // NamedProvider impl
        impl<F: Sync + Send, S: Sync + Send, T: Sync + Send> ::webwire::NamedProvider<S> for #provider_name<F>
        where
            F: Fn(::std::sync::Arc<S>) -> T,
            T: #service_name + 'static,
        {
            const NAME: &'static str = #service_name_str;
        }
        // Provider impl
        impl<F: Sync + Send, S: Sync + Send, T: Sync + Send> ::webwire::Provider<S> for #provider_name<F>
        where
            F: Fn(::std::sync::Arc<S>) -> T,
            T: #service_name + 'static,
        {
            fn call(
                &self,
                session: &::std::sync::Arc<S>,
                _service: &str,
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

fn gen_provider_matches(service: &schema::Service, ns: &[String]) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods.iter() {
        let name = quote::format_ident!("{}", method.name);
        let name_str = &method.name;
        let input = match &method.input {
            Some(type_) => gen_typeref(type_, ns),
            None => quote! { () },
        };
        /*
        let output = match &method.output {
            Some(type_) => gen_typeref(type_),
            None => quote! { () },
        };
        */
        let method_call = match &method.input {
            None => quote! {
                let output = service.#name().await.map_err(|e| e.into())?;
            },
            Some(type_) => {
                let validation = if type_.is_scalar() {
                    quote! {}
                } else {
                    quote! {
                        ::validator::Validate::validate(&input).map_err(::webwire::ProviderError::ValidationError)?;
                    }
                };
                quote! {
                    let input = serde_json::from_slice::<#input>(&input)
                            .map_err(::webwire::ProviderError::DeserializerError)?;
                    #validation
                    let output = service.#name(&input).await.map_err(|e| e.into())?;
                }
            }
        };
        stream.extend(quote! {
            #name_str => Box::pin(async move {
                #method_call
                let response = serde_json::to_vec(&output)
                    .map_err(|e| ::webwire::ProviderError::SerializerError(e))
                    .map(::bytes::Bytes::from)?;
                Ok(response)
            }),
        });
    }
    stream
}

fn gen_consumer(service: &schema::Service, ns: &[String]) -> TokenStream {
    let consumer_name = quote::format_ident!("{}Consumer", service.name);
    let consumer_methods = gen_consumer_methods(service, ns);
    quote! {
        pub struct #consumer_name<'a>(pub &'a (dyn ::webwire::Consumer + ::std::marker::Sync + ::std::marker::Send));
        impl<'a> #consumer_name<'a> {
            #consumer_methods
        }
    }
}

fn gen_consumer_methods(service: &schema::Service, ns: &[String]) -> TokenStream {
    let mut stream = TokenStream::new();
    let service_name_str = if ns.is_empty() {
        service.name.to_owned()
    } else {
        format!("{}.{}", ns.join("."), &service.name)
    };
    for method in service.methods.iter() {
        let signature = gen_consumer_method_signature(method, ns);
        let method_name_str = &method.name;
        let serialization = match method.input {
            Some(_) => quote! {
                let data: ::bytes::Bytes = serde_json::to_vec(input)
                    .map_err(|e| ::webwire::ConsumerError::SerializerError(e))?
                    .into();
            },
            None => quote! {
                let data = ::bytes::Bytes::new();
            },
        };
        stream.extend(quote! {
            #signature {
                #serialization
                let output = self.0.request(#service_name_str, #method_name_str, data).await?;
                let response = ::serde_json::from_slice(&output)
                    .map_err(|e| ::webwire::ConsumerError::DeserializerError(e))?;
                Ok(response)
            }
        })
    }
    stream
}

fn gen_consumer_method_signature(method: &schema::Method, ns: &[String]) -> TokenStream {
    let name = quote::format_ident!("{}", method.name);
    let input_arg = match &method.input {
        Some(type_) => {
            let input_type = gen_typeref(type_, ns);
            quote! { input: & #input_type }
        }
        None => quote! {},
    };
    let output = match &method.output {
        Some(type_) => gen_typeref(type_, ns),
        None => quote! { () },
    };
    quote! {
        pub async fn #name(&self, #input_arg) -> Result<#output, ::webwire::ConsumerError>
    }
}

fn gen_typeref(type_: &schema::Type, ns: &[String]) -> TokenStream {
    match type_ {
        schema::Type::None => quote! { () },
        schema::Type::Boolean => quote! { bool },
        schema::Type::Integer => quote! { i64 },
        schema::Type::Float => quote! { f64 },
        schema::Type::String => quote! { String },
        schema::Type::UUID => quote! { ::uuid::Uuid },
        schema::Type::Date => quote! { ::chrono::Date },
        schema::Type::Time => quote! { ::chrono::Time },
        schema::Type::DateTime => quote! { ::chrono::DateTime<::chrono::Utc> },
        schema::Type::Option(some) => {
            let some_type = gen_typeref(some, ns);
            quote! { std::option::Option<#some_type> }
        }
        schema::Type::Result(ok, err) => {
            let ok_type = gen_typeref(ok, ns);
            let err_type = gen_typeref(err, ns);
            quote! { std::result::Result<#ok_type, #err_type> }
        }
        // complex types
        schema::Type::Array(array) => {
            let item_type = gen_typeref(&array.item_type, ns);
            quote! {
                std::vec::Vec<#item_type>
            }
        }
        schema::Type::Map(map) => {
            let key_type = gen_typeref(&map.key_type, ns);
            let value_type = gen_typeref(&map.value_type, ns);
            quote! {
                std::collections::HashMap<#key_type, #value_type>
            }
        }
        // named
        schema::Type::Ref(typeref) => gen_typeref_ref(typeref, ns),
        schema::Type::Builtin(name) => {
            // FIXME unwrap... igh!
            let identifier: TokenStream = ::syn::parse_str(name).unwrap();
            quote! {
                #identifier
            }
        }
    }
}

fn gen_typeref_ref(typeref: &TypeRef, ns: &[String]) -> TokenStream {
    let mut generics_stream = TokenStream::new();
    if !typeref.generics().is_empty() {
        for generic in typeref.generics().iter() {
            let type_ = gen_typeref(generic, ns);
            generics_stream.extend(quote! {
                #type_,
            })
        }
        generics_stream = quote! {
            < #generics_stream >
        }
    }
    let typeref_fqtn = typeref.fqtn();
    let common_ns = typeref_fqtn
        .ns
        .iter()
        .zip(ns.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let relative_ns = ns[common_ns..]
        .iter()
        .map(|_| quote::format_ident!("super"))
        .chain(
            typeref_fqtn.ns[common_ns..]
                .iter()
                .map(|x| quote::format_ident!("{}", x)),
        )
        .fold(TokenStream::new(), |mut stream, name| {
            let name = quote::format_ident!("{}", name);
            stream.extend(quote! { #name :: });
            stream
        });
    // FIXME fqtn
    match &*typeref_fqtn.name {
        // FIXME `None` should be made into a buitlin type
        "None" => quote! { () },
        name => {
            let name = quote::format_ident!("{}", name);
            quote! {
                #relative_ns #name #generics_stream
            }
        }
    }
}
