mod data_enum;
mod data_struct;

use crate::util::parse::AttributeMap;
use proc_macro2::TokenStream;
use syn::Ident;

pub trait DeriveRedisValue {
    fn derive_redis_value(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream;
}
