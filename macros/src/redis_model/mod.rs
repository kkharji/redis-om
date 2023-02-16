use crate::util::parse::AttributeMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DataStruct, Ident};

pub trait DeriveRedisModel {
    fn derive_redis_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream;
}

impl DeriveRedisModel for DataStruct {
    fn derive_redis_model(&self, ident: &Ident, _attrs: &AttributeMap) -> TokenStream {
        // TODO: introduce an option to set redis_key

        quote! {
            impl ::redis_om::RedisModel for #ident {
                fn redis_key() -> &'static str {
                    stringify!(#ident)
                }

                fn _get_primary_key(&self) -> &str {
                    // TODO: Require struct to have either id or pk key
                    &self.id
                }

                fn _set_primary_key(&mut self, pk: String) {
                    // TODO: Require struct to have either id or pk key
                    self.id = pk;
                }
            }
        }
    }
}

impl DeriveRedisModel for DataEnum {
    fn derive_redis_model(&self, _ident: &Ident, _attrs: &AttributeMap) -> TokenStream {
        quote!()
    }
}
