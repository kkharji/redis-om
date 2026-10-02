use crate::ast::{AccumulatorExt, Container, FieldAttr, style_name};
use crate::type_ext::TypeExt;
use darling::ast::{Data, Style};
use darling::error::Accumulator;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Type;

pub fn derive(errors: &mut Accumulator, cont: &Container) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let mut stream = TokenStream::new();
    crate::redis_model::derive(errors, cont)?.to_tokens(&mut stream);
    redis_schema::derive(errors, cont)?.to_tokens(&mut stream);

    Ok(quote! {
        #stream
        impl ::redis_om::JsonModel for #type_name { }
    })
}

mod redis_schema {
    use super::*;

    pub fn derive(errors: &mut Accumulator, cont: &Container) -> Result<TokenStream, ()> {
        let type_name = cont.ident;
        let prefix_key = cont.prefix_key.as_str();

        let Data::Struct(fields) = &cont.data else {
            let msg = &"Enum is not currently supported for redissearch_model";
            errors.push_spanned_error(cont.ident, msg);
            return Err(());
        };

        let Style::Struct = fields.style else {
            let msg = format!(
                "{} Struct is not supported",
                style_name(fields.style, fields.fields.len())
            );
            errors.push_spanned_error(cont.ident, msg);
            return Err(());
        };

        let redis_search_schema = format!(
            "ON JSON PREFIX 1 {prefix_key} SCHEMA {}",
            fields
                .iter()
                .map(|field| {
                    let mut schema_parts = Vec::new();
                    let name = &field.attrs.name.serialize;
                    let json_path = "$";

                    if field.attrs.primary_key {
                        schema_parts.push(format!("{json_path}.{name} AS {name} TAG SEPARATOR |"));
                    } else if field.attrs.index || field.ty.is_list_collection() {
                        schema_parts.push(schema_for_type(
                            json_path,
                            name,
                            &field.attrs,
                            &field.ty,
                        ));
                    }

                    schema_parts.join(" ")
                })
                .collect::<Vec<_>>()
                .join(" ")
                .trim()
        );

        Ok(quote! {
            impl ::redis_om::RedisSearchModel for #type_name {
                const _REDIS_SEARCH_SCHEMA: &'static str = #redis_search_schema;
            }
        })
    }

    fn schema_for_type(json_path: &str, name: &str, attrs: &FieldAttr, ty: &Type) -> String {
        let mut schema: Vec<String> = vec![];
        let path = format!("{json_path}.{name}");

        if ty.is_list_collection()
            && let Some(inner) = ty.get_inner_type()
        {
            schema.push(schema_for_type(json_path, name, attrs, inner));
        } else if ty.is_numeric_type() {
            schema.push(format!("{path} AS {name} NUMERIC"));
        } else if ty.is_ident("String") {
            if attrs.fts {
                schema.push(format!(
                    "{path} AS {name} TAG SEPARATOR | {path} AS {name}_fts TEXT"
                ));
            } else {
                schema.push(format!("{path} AS {name} TAG SEPARATOR |"));
            }
        } else {
            schema.push(format!("{path} AS {name} TAG SEPARATOR |"))
        }

        if attrs.sortable {
            schema.push("SORTABLE".into())
        }

        schema.join(" ")
    }
}
