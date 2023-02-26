use crate::ast::{Container, Ctx};
use crate::ext::TypeExt;
use crate::util::parse::{self, AttributeMap};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{DataStruct, Field, Fields, Ident, Type};

use super::Derive;

pub fn derive(ctx: &Ctx, cont: &Container) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let mut stream = TokenStream::new();
    crate::redis_model::derive(ctx, cont)?.to_tokens(&mut stream);
    crate::redissearch_model::derive(ctx, cont, Derive::JsonModel)?.to_tokens(&mut stream);

    quote! {
        impl ::redis_om::JsonModel for #type_name { }
    }
    .to_tokens(&mut stream);

    Ok(stream)
}
