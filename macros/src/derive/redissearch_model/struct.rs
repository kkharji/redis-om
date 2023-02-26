use crate::{
    ast::{Container, Ctx, Field, FieldAttr, Style},
    derive::Derive,
    ext::TypeExt,
};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{AttrStyle, Type};

pub(super) fn derive(
    ctx: &Ctx,
    cont: &Container,
    model_type: Derive,
    style: &Style,
    fields: &[Field],
) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let prefix_key = cont.attrs.prefix_key.as_str();
    let leading = match model_type {
        Derive::HashModel => "ON HASH PREFIX 1",
    };

    let fields_schema = fields
        .iter()
        .map(|field| {
            let key = field.attrs.name.serialize_name();
            let mut schema_parts = Vec::new();
            let Field { attrs, ty, .. } = field;

            if attrs.primary_key {
                let value = format!("{key} TAG SEPARATOR |");
                schema_parts.push(value);
            } else if attrs.index {
                let value = schema_for_type(attrs, &ty);
                schema_parts.push(value);
            } else if ty.is_list_collection() {
                let ty = &ty.get_inner_type().expect("inner type of list-like type");
                let value = schema_for_type(attrs, &ty);
                schema_parts.push(value);
            }
            schema_parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join(" ");

    let redis_search_schema = format!("{leading} {prefix_key} SCHEMA {fields_schema}");

    // TODO: Find a way to ignore types already implements default trait.
    match style {
        Style::Struct => Ok(quote! {
            impl ::redis_om::RedisSearchModel for #type_name {
                const REDIS_SEARCH_SCHEMA: &'static str = #redis_search_schema;
            }
        }),
        Style::Tuple | Style::Newtype | Style::Unit => {
            let msg = format!("{:?} Struct is not supported", style);
            ctx.error_spanned_by(cont.original, msg);
            Err(())
        }
    }
}

fn schema_for_type(attrs: &FieldAttr, ty: &Type) -> String {
    let mut schema: Vec<String> = vec![];
    let name = attrs.name.serialize_name();

    // TODO: Support embedded Redis Model

    let sortable = attrs.sortable;

    if ty.is_list_collection() {
        schema.push(schema_for_type(
            attrs,
            ty.get_inner_type()
                .expect("Getting inner type for list like collection"),
        ));
    } else if ty.is_numeric_type() {
        schema.push(format!("{name} NUMERIC"));
    } else if ty.is_ident("String") {
        if attrs.fts {
            schema.push(format!("{name} TAG SEPARATOR | {name} AS {name}_fts TEXT"));
        } else {
            schema.push(format!("{name} TAG SEPARATOR |"));
        }
    } else {
        schema.push(format!("{name} TAG SEPARATOR |"))
    }

    if sortable {
        schema.push("SORTABLE".into())
    }

    schema.join(" ")
}
