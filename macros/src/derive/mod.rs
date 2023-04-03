use syn::LitStr;

use crate::ast::Ctx;

pub mod get_set;
pub mod hash_model;
#[cfg(feature = "json")]
pub mod json_model;
pub mod redis_model;
pub mod stream_model;
pub mod value;

#[derive(Clone, Copy)]
pub enum Derive {
    HashModel,
    JsonModel,
}

impl Derive {
    pub fn from_lit_str(ctx: &Ctx, litstr: &LitStr) -> Result<Self, ()> {
        let res = match litstr.value().as_str() {
            "HashModel" => Self::HashModel,
            _ => {
                ctx.error_spanned_by(litstr, "Unrecognized model type");
                return Err(());
            }
        };

        Ok(res)
    }
}
