use crate::ast::{symbols::*, Ctx, Symbol, VecAttr};
use syn::punctuated::Punctuated;
use syn::{LitStr, Meta, MetaList, NestedMeta, Token};

use super::LitExt;

type SerAndDe<T> = (Option<T>, Option<T>);

pub trait MetaListExt<'a> {
    fn get_renames(&self, cx: &Ctx) -> Result<SerAndDe<&LitStr>, ()>;
    fn get_multiple_renames(&self, cx: &Ctx) -> Result<(Option<&LitStr>, Vec<&LitStr>), ()>;
}

impl<'a> MetaListExt<'a> for MetaList {
    fn get_renames(&self, cx: &Ctx) -> Result<SerAndDe<&LitStr>, ()> {
        let (ser, de) = get_ser_and_de(
            cx,
            RENAME,
            &self.nested,
            |cx: &Ctx, a: Symbol, m: Symbol, l: &syn::Lit| l.to_lit_str_0(cx, a, m),
        )?;
        Ok((ser.at_most_one()?, de.at_most_one()?))
    }

    fn get_multiple_renames(&self, cx: &Ctx) -> Result<(Option<&LitStr>, Vec<&LitStr>), ()> {
        let (ser, de) = get_ser_and_de(
            cx,
            RENAME,
            &self.nested,
            |cx: &Ctx, a: Symbol, m: Symbol, l: &syn::Lit| l.to_lit_str_0(cx, a, m),
        )?;
        Ok((ser.at_most_one()?, de.get()))
    }
}

fn get_ser_and_de<'a, 'b, T, F>(
    ctx: &'b Ctx,
    attr_name: Symbol,
    metas: &'a Punctuated<syn::NestedMeta, Token![,]>,
    f: F,
) -> Result<(VecAttr<'b, T>, VecAttr<'b, T>), ()>
where
    T: 'a,
    F: Fn(&Ctx, Symbol, Symbol, &'a syn::Lit) -> Result<T, ()>,
{
    let mut ser_meta = VecAttr::new(ctx, attr_name);
    let mut de_meta = VecAttr::new(ctx, attr_name);

    for meta in metas {
        match meta {
            NestedMeta::Meta(Meta::NameValue(meta)) if meta.path == SERIALIZE => {
                if let Ok(v) = f(ctx, attr_name, SERIALIZE, &meta.lit) {
                    ser_meta.insert(&meta.path, v);
                }
            }
            NestedMeta::Meta(Meta::NameValue(meta)) if meta.path == DESERIALIZE => {
                if let Ok(v) = f(ctx, attr_name, DESERIALIZE, &meta.lit) {
                    de_meta.insert(&meta.path, v);
                }
            }
            _ => {
                let msg = format!(
                    "malformed {0} attribute, expected `{0}(serialize = ..., deserialize = ...)`",
                    attr_name
                );

                ctx.error_spanned_by(meta, msg);
                return Err(());
            }
        }
    }
    Ok((ser_meta, de_meta))
}
