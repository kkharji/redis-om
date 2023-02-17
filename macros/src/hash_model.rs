use crate::{
    generate, redis_model::DeriveRedisModel, util::parse::AttributeMap, value::DeriveRedisValue,
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{DataStruct, Fields, Ident};

pub trait DeriveHashModel {
    fn derive_hash_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream;
}

impl DeriveHashModel for DataStruct {
    fn derive_hash_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream {
        let mut stream = TokenStream::new();
        let Fields::Named(fields) = &self.fields else { panic!("tuple and unit structs are not supported"); };

        let functions = fields
            .named
            .iter()
            .map(generate::common_get_set)
            .collect::<Vec<_>>();

        // TODO: Support generics and where clause
        quote::quote! {
            #[allow(dead_code)]
            #[allow(clippy::all)]
            impl #ident {
                #(#functions)*
            }
        }
        .to_tokens(&mut stream);

        // TODO: make sure ident haven't already implemented redis::ToRedisArgs and redis::FromRedisValue
        self.derive_redis_value(ident, attrs).to_tokens(&mut stream);
        self.derive_redis_model(ident, attrs).to_tokens(&mut stream);

        stream.into()
    }
}
