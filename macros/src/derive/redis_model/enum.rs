use super::TokenStream;
use crate::ast::Variant;
use crate::ast::{Container, Ctx, Style};
use crate::util::{parse::AttributeMap, string};
use quote::quote;
use syn::{DataEnum, Fields, Ident};

pub(super) fn derive(ctx: &Ctx, cont: &Container, variants: &[Variant]) -> Result<TokenStream, ()> {
    ctx.error_spanned_by(cont.original, "RedisModel is only supported for structs");
    return Err(());
}
