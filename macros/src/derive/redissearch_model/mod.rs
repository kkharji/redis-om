// mod r#enum;
mod r#struct;

use crate::ast::{Container, Ctx, Data};
use proc_macro2::TokenStream;
use quote::quote;

use super::Derive;

pub fn derive(ctx: &Ctx, cont: &Container, model_type: Derive) -> Result<TokenStream, ()> {
    match &cont.data {
        Data::Struct(style, fields) => r#struct::derive(ctx, cont, model_type, style, fields),
        Data::Enum(variants) => {
            ctx.error_spanned_by(cont.ident, "Enum is not supported for redissearch_model");
            Err(())
        }
    }
}
