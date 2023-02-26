use crate::ast::{Container, Ctx, Field, Style};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::AttrStyle;

pub(super) fn derive(
    ctx: &Ctx,
    cont: &Container,
    style: &Style,
    fields: &[Field],
) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let prefix_key = cont.attrs.prefix_key.as_str();

    // TODO: Find away to ignore types already implements default trait.
    match style {
        Style::Struct => {
            let pk_field = fields.iter().find(|f| f.attrs.primary_key);
            let pk_ident = pk_field
                .map(|v| v.ident.unwrap().to_owned())
                .unwrap_or_else(|| Ident::new("id", cont.ident.span()));

            if fields.iter().find(|f| f.ident == Some(&pk_ident)).is_none() {
                let msg = format!("A primary field doesn't exists, either add `id` field or annotate a field `primary_key`");
                ctx.error_spanned_by(cont.original, msg);
                return Err(());
            };

            Ok(quote! {
                impl ::redis_om::RedisModel for #type_name {
                    fn _prefix_key() -> &'static str {
                        #prefix_key
                    }

                    fn _get_pk(&self) -> &str {
                        &self.#pk_ident
                    }

                    fn _set_pk(&mut self, pk: String) {
                        self.#pk_ident = pk;
                    }
                }
            })
        }
        Style::Tuple | Style::Newtype | Style::Unit => {
            let msg = format!("{:?} Struct is not supported", style);
            ctx.error_spanned_by(cont.original, msg);
            Err(())
        }
    }
}
