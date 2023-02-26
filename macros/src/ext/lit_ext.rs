use syn::{
    parse::{self, Parse},
    LitStr,
};

use crate::{
    ast::{Ctx, Symbol},
    util::respan,
};

pub trait LitExt {
    fn to_string(&self) -> Option<String>;
    fn to_lit_str(&self, ctx: &Ctx, name: Symbol) -> Result<&LitStr, ()>;
    fn to_lit_str_0(
        &self,
        cx: &Ctx,
        attr_name: Symbol,
        meta_item_name: Symbol,
    ) -> Result<&syn::LitStr, ()>;
    fn to_type<T: Parse>(&self, ctx: &Ctx, name: Symbol) -> Result<T, ()>;
    fn to_expr_path(&self, ctx: &Ctx, name: Symbol) -> Result<syn::ExprPath, ()>;
}

impl LitExt for syn::Lit {
    fn to_string(&self) -> Option<String> {
        match self {
            syn::Lit::Str(ref s) => Some(s.value()),
            _ => None,
        }
    }

    fn to_lit_str_0(&self, ctx: &Ctx, aname: Symbol, mname: Symbol) -> Result<&syn::LitStr, ()> {
        if let syn::Lit::Str(lit) = self {
            Ok(lit)
        } else {
            ctx.error_spanned_by(
                self,
                format!("expected redis {aname} attribute to be a string: `{mname} = \"...\"`"),
            );

            Err(())
        }
    }

    fn to_lit_str(&self, ctx: &Ctx, name: Symbol) -> Result<&LitStr, ()> {
        self.to_lit_str_0(ctx, name, name)
    }

    fn to_type<T: Parse>(&self, ctx: &Ctx, name: Symbol) -> Result<T, ()> {
        let lit_str = self.to_lit_str(ctx, name)?;
        LitStrExt::parse(lit_str).map_err(|_| {
            ctx.error_spanned_by(self, format!("failed to parse: {:?}", lit_str.value()))
        })
    }

    fn to_expr_path(&self, ctx: &Ctx, name: Symbol) -> Result<syn::ExprPath, ()> {
        self.to_type(ctx, name)
    }
}

pub trait LitStrExt {
    fn parse<T: Parse>(&self) -> parse::Result<T>;
}

impl LitStrExt for LitStr {
    fn parse<T: Parse>(&self) -> parse::Result<T> {
        let stream = syn::parse_str(&self.value())?;
        let tokens = respan(stream, self.span());
        syn::parse2(tokens)
    }
}
