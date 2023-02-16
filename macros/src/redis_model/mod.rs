use crate::util::parse::AttributeMap;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataStruct, Ident};

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

        let pk_field = attrs
            .get("pk_field")
            .map(ToString::to_string)
            .unwrap_or_else(|| "id".into());

        let pk_field_ident = Ident::new(&pk_field, ident.span());

        let pk_field_exists = self.fields.iter().any(|field| {
            field
                .ident
                .as_ref()
                .map(|ident| ident == &pk_field_ident)
                .unwrap_or_default()
        });

        if !pk_field_exists {
            // Add pk_field_name as a new field to the struct
            panic!("Model requires id field or pk_field to be set!!",);
        }

        quote! {
            impl ::redis_om::RedisModel for #ident {
                fn redis_key() -> &'static str {
                    #key_prefix
                }

                fn _get_primary_key(&self) -> &str {
                    &self.#pk_field_ident
                }

                fn _set_primary_key(&mut self, pk: String) {
                    self.#pk_field_ident = pk;
                }
            }
        }
    }
}
