use crate::{redis_model::DeriveRedisModel, util::parse::AttributeMap, value::DeriveRedisValue};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{DataStruct, Ident};

pub trait DeriveHashModel {
    fn derive_hash_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream;
}

impl DeriveHashModel for DataStruct {
    fn derive_hash_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream {
        let mut stream = TokenStream::new();
        // TODO: make sure ident haven't already implemented redis::ToRedisArgs and
        // redis::FromRedisValue
        self.derive_redis_value(ident, attrs).to_tokens(&mut stream);
        self.derive_redis_model(ident, attrs).to_tokens(&mut stream);

        stream.into()
    }
}
