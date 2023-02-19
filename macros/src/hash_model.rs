use crate::{
    ext::TypeExt,
    generate,
    redis_model::DeriveRedisModel,
    util::parse::{self, AttributeMap},
    value::DeriveRedisValue,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataStruct, Field, Fields, Ident, Type};

pub trait DeriveHashModel {
    fn derive_hash_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream;
}

impl DeriveHashModel for DataStruct {
    fn derive_hash_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream {
        let mut stream = TokenStream::new();
        let Fields::Named(fields) = &self.fields else { panic!("tuple and unit structs are not supported for hash models"); };

        // TODO: make sure ident haven't already implemented redis::ToRedisArgs and redis::FromRedisValue
        impl_getters_setters(fields, ident).to_tokens(&mut stream);
        self.derive_redis_value(ident, attrs).to_tokens(&mut stream);
        self.derive_redis_model(ident, attrs).to_tokens(&mut stream);

        let redissearch_schema = generate_redissearch_schema(ident, attrs, fields);

        quote! {
            impl HashModel for #ident {
                fn redissearch_schema() -> &'static str {
                    #redissearch_schema
                }
            }
        }
        .to_tokens(&mut stream);

        stream.into()
    }
}

fn generate_redissearch_schema(
    ident: &Ident,
    attrs: &AttributeMap,
    fields: &syn::FieldsNamed,
) -> String {
    let key_prefix = attrs
        .get("key_prefix")
        .map(|s| quote!(#s))
        .unwrap_or_else(|| quote!(#ident));
    let pk_field = attrs
        .get("pk_field")
        .map(ToString::to_string)
        .unwrap_or_else(|| "id".into());
    let pk_field_ident = Ident::new(&pk_field, ident.span());
    let schema_parts = fields
        .named
        .iter()
        .map(|f| schema_for_field(f, &pk_field_ident))
        .collect::<Vec<_>>();

    format!(
        "ON HASH PREFIX 1 {key_prefix} SCHEMA {}",
        schema_parts.join(" ")
    )
}

fn schema_for_field(field: &Field, pk_field_ident: &Ident) -> String {
    let field_ident = field.ident.as_ref().unwrap();
    let attrs = parse::attributes(&field.attrs);
    let mut schema_parts = Vec::new();

    if field_ident == pk_field_ident {
        schema_parts.push(format!("{field_ident} TAG SEPARATOR |"));
    } else if attrs.get("index").map(|v| v == "true").unwrap_or_default() {
        schema_parts.push(schema_for_type(field_ident, &field.ty, &attrs));
    } else if field.ty.is_list_collection() {
        schema_parts.push(schema_for_type(
            field_ident,
            &field
                .ty
                .get_inner_type()
                .expect("Inner type of list-like type"),
            &attrs,
        ));
    };

    schema_parts.join(" ")
}

fn schema_for_type(ident: &Ident, ty: &Type, attrs: &AttributeMap) -> String {
    let mut schema: Vec<String> = vec![];
    let name = ident.to_string();

    // TODO: Support embedded Redis Model

    let sortable = attrs
        .get("sortable")
        .map(|v| v == "true")
        .unwrap_or_default();

    if ty.is_list_collection() {
        schema.push(schema_for_type(
            ident,
            ty.get_inner_type()
                .expect("Getting inner type for list like collection"),
            attrs,
        ));
    } else if ty.is_numeric_type() {
        schema.push(format!("{name} NUMERIC"));
    } else if ty.is_ident("String") {
        let full_text_search = attrs
            .get("full_text_search")
            .map(|v| v == "true")
            .unwrap_or_default();
        if full_text_search {
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

fn impl_getters_setters(fields: &syn::FieldsNamed, ident: &Ident) -> TokenStream {
    let functions = fields
        .named
        .iter()
        .map(generate::common_get_set)
        .collect::<Vec<_>>();

    // TODO: Support generics and where clause
    quote! {
        #[allow(dead_code)]
        #[allow(clippy::all)]
        impl #ident {
            #(#functions)*
        }
    }
}
