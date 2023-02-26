#![allow(unused)]
//! Derive proc macros for redis-om crate
#![deny(missing_docs, unstable_features)]

mod ast;
mod derive;
mod ext;
mod util;

use ast::{Container, Ctx};
use derive::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data::*, DeriveInput};

#[proc_macro_derive(HashModel, attributes(redis))]
/// ....
pub fn hash_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(hash_model::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

#[cfg(feature = "json")]
#[proc_macro_derive(JsonModel, attributes(redis))]
/// ....
pub fn json_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(json_model::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

#[proc_macro_derive(RedisModel, attributes(redis))]
/// ....
pub fn redis_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(redis_model::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

#[proc_macro_derive(RedisSearchModel, attributes(redis))]
/// ....
pub fn redis_search_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(
            redissearch_model::derive(&ctx, &cont, cont.attrs.model_type),
            ctx,
        ),
        Err(value) => return value,
    }
}

/// Derive procedural macro that automatically generate implementation for
///
/// [`RedisTransportValue`](redis_om::RedisTransportValue), which requires implementation of the following:
///
/// - [`ToRedisArgs`](redis::ToRedisArgs),
/// - [`FromRedisValue`](redis::FromRedisValue)
///
/// # Example
///
/// ```
/// #[derive(redis_om::RedisTransportValue)]
/// struct Test {
///     #[redis(primary_key)] // required if no `id` field exists
///     field_one: String,
///     field_two: i32,
///     field_three: Vec<u8>,
/// }
/// ```
///
/// # Attributes
///
/// ## `rename_all = "some_case"`
///
/// This attribute sets the default casing to be used for field names when generating Redis command arguments.
/// Possible values are:
///
/// - `"snake_case"`: field names will be converted to `snake_case`
/// - `"kebab-case"`: field names will be converted to `kebab-case`
/// - `"camelCase"`: field names will be converted to `camelCase`
/// - `"PascalCase"`: field names will be converted to `PascalCase`
///
/// This attribute can be overridden for individual fields using the `rename` attribute.
///
/// ## `rename = "some_field"`
///
/// This attribute sets the Redis key name to be used for a field when generating Redis command arguments.
/// This attribute takes precedence over the `rename_all` attribute.
///
/// # Restrictions
///
/// - Enums with fields are not supported
/// - Generics are not supported
/// - Public fields are required
/// - Fields with types that do not implement `ToRedisArgs` and `FromRedisValue` are not supported
///
#[proc_macro_derive(RedisTransportValue, attributes(redis))]
pub fn redis_transport_value(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(value::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

// --------------------------------------------------------------------------------------
// --------------------------------------------------------------------------------------

fn into_stream(stream: Result<proc_macro2::TokenStream, ()>, ctx: Ctx) -> TokenStream {
    let check = ctx.check();
    let res = if check.is_err() {
        into_to_compile_errors(check.unwrap_err())
    } else {
        stream.unwrap_or_else(|_| into_to_compile_errors(check.unwrap_err()))
    };
    res.into()
}

fn into_to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}

fn into_container(input: &DeriveInput) -> Result<(Ctx, Container<'_>), TokenStream> {
    let ctx = Ctx::new();
    let cont = match Container::new(&ctx, input) {
        Some(cont) => cont,
        None => return Err(into_to_compile_errors(ctx.check().unwrap_err()).into()),
    };
    Ok((ctx, cont))
}
