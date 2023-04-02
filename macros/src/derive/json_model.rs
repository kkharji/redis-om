use crate::ast::{Container, Ctx, Data, Field, FieldAttr, Style};
use crate::ext::{AttributeExt, TypeExt};
use crate::util::parse::{self, AttributeMap};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataStruct, Ident, Type};

use super::Derive;

pub fn derive(ctx: &Ctx, cont: &Container) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let mut stream = TokenStream::new();
    crate::redis_model::derive(ctx, cont)?.to_tokens(&mut stream);
    redis_schema::derive(ctx, cont)?.to_tokens(&mut stream);
    let mut attributes = Vec::<syn::Attribute>::new();
    #[cfg(feature = "aio")]
    attributes.push(syn::Attribute::from_token_stream(quote!(#[::redis_om::async_trait])).unwrap());

    Ok(quote! {
        #stream
        #(#attributes)*
        impl ::redis_om::JsonModel for #type_name { }
    })
}

mod redis_schema {
    use super::*;

    pub fn derive(ctx: &Ctx, cont: &Container) -> Result<TokenStream, ()> {
        let type_name = cont.ident;
        let prefix_key = cont.attrs.prefix_key.as_str();

        let Data::Struct(style, fields) = &cont.data else {
            let msg = &"Enum is not currenlty supported for redissearch_model";
            ctx.error_spanned_by(cont.ident, msg);
            return Err(());
        };

        let Style::Struct = style else {
            let msg = format!("{:?} Struct is not supported", style);
            ctx.error_spanned_by(cont.original, msg);
            return Err(());
        };

        let redis_search_schema = format!(
            "ON JSON PREFIX 1 {prefix_key} SCHEMA {}",
            fields
                .iter()
                .map(schema_for_field)
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

    // TODO: Support embedded Redis Model
    fn schema_for_field(field: &Field) -> String {
        let mut schema_parts = Vec::new();
        let name = field.attrs.name.serialize_name();
        let json_path = "$";

        if field.attrs.primary_key {
            let value = format!("{json_path}.{name} AS {name} TAG SEPARATOR |");
            schema_parts.push(value);
        } else if field.attrs.index {
            let value = schema_for_type(json_path, name, &field.attrs, field.ty);
            schema_parts.push(value);
        } else if field.ty.is_list_collection() {
            let ty = field
                .ty
                .get_inner_type()
                .expect("inner type of list-like type");
            let value = schema_for_type(json_path, name, &field.attrs, field.ty);
            schema_parts.push(value);
        }

        schema_parts.join(" ")
    }

    fn schema_for_type(json_path: &str, name: String, attrs: &FieldAttr, ty: &Type) -> String {
        let mut schema: Vec<String> = vec![];
        let path = format!("{json_path}.{name}");

        if ty.is_list_collection() {
            let ty = &ty.get_inner_type().unwrap();
            schema.push(schema_for_type(json_path, name, attrs, ty));
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
