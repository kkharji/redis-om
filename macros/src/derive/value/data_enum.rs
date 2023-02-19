use super::{DeriveRedisValue, TokenStream};
use crate::util::{parse::AttributeMap, string};
use quote::quote;
use syn::{DataEnum, Fields, Ident};

impl DeriveRedisValue for DataEnum {
    fn derive_redis_value(&self, ident: &Ident, attrs: &AttributeMap) -> TokenStream {
        let variants = &self.variants;

        if !variants.iter().all(|v| matches!(v.fields, Fields::Unit)) {
            panic!("Only Enums without fields are supported");
        }

        let casing = attrs.get("rename_all").map(|v| v.as_str());
        let (values, (to_redis_match, from_redis_match)): (Vec<_>, (Vec<_>, Vec<_>)) = variants
            .iter()
            .map(|v| {
                let value = string::transform_casing(&v.ident.to_string(), casing);
                let name = &v.ident;
                let to_redis = quote!(#ident::#name => { out.write_arg(#value.as_bytes()); },);
                let from_redis = quote!(#value => Ok(#ident::#name),);
                (value.to_owned(), (to_redis, from_redis))
            })
            .unzip();

        let variants_str = values.join(", ");

        quote! {
            impl redis::ToRedisArgs for #ident {
                fn write_redis_args<W: ?Sized + redis::RedisWrite>(&self, out: &mut W) {
                    match self { #(#to_redis_match)* }
                }
            }

            impl redis::FromRedisValue for #ident {
                fn from_redis_value(v: &redis::Value) -> Result<Self, redis::RedisError> {
                    use redis::{ErrorKind::TypeError, Value};

                    let Value::Data(data) = v else {
                        let msg = format!("{:?}", v);
                        return Err((TypeError, "Expected Redis string, got:", msg).into());
                    };

                    let value = std::str::from_utf8(&data[..])?;

                    match value {
                        #(#from_redis_match)*
                        v => {
                            let msg = format!("{}, Expected one of: {}", v, #variants_str);
                            Err((TypeError, "Invalid enum variant:", msg).into())
                        },
                    }
                }
            }
        }
    }
}
