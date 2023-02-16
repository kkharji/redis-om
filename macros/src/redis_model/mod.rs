use crate::util::parse::AttributeMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, DataStruct, Ident};

pub trait DeriveRedisModel {
    fn derive_redis_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream;
}

impl DeriveRedisModel for DataStruct {
    fn derive_redis_model(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream {
        // TODO: introduce an option to set redis_key

        let key_prefix = attrs
            .get("key_prefix")
            .map(|s| quote!(#s))
            .unwrap_or_else(|| quote!(stringify!(#ident)));

        let pk_field_name = attrs
            .get("pk_field")
            .map(|s| {
                let ident = Ident::new(s, ident.span());
                quote!(#ident)
            })
            .unwrap_or_else(|| quote!(id));

        quote! {
            impl ::redis_om::RedisModel for #ident {
                fn redis_key() -> &'static str {
                    #key_prefix
                }

                fn _get_primary_key(&self) -> &str {
                    &self.#pk_field_name
                }

                fn _set_primary_key(&mut self, pk: String) {
                    self.#pk_field_name = pk;
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
