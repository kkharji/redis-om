mod r#enum;
mod r#struct;

use crate::ast::{Container, Ctx, Data};
use proc_macro2::TokenStream;
use quote::quote;

pub fn derive(ctx: &Ctx, cont: &Container) -> Result<TokenStream, ()> {
    match &cont.data {
        Data::Enum(variants) => r#enum::derive(ctx, cont, variants),
        Data::Struct(style, fields) => r#struct::derive(ctx, cont, style, fields),
    }
}
