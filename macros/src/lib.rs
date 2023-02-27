#![allow(unused)]
//! Derive proc macros for redis-om crate
#![deny(unstable_features)]

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
pub fn hash_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(hash_model::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

#[cfg(feature = "json")]
#[proc_macro_derive(JsonModel, attributes(redis))]
pub fn json_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(json_model::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

#[proc_macro_derive(RedisModel, attributes(redis))]
pub fn redis_model(attr: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as DeriveInput);
    match into_container(&input) {
        Ok((ctx, cont)) => into_stream(redis_model::derive(&ctx, &cont), ctx),
        Err(value) => return value,
    }
}

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
