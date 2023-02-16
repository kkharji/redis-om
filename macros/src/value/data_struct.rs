use super::DeriveRedisValue;
use crate::util::{parse, string};
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{DataStruct, Fields, Ident};

impl DeriveRedisValue for DataStruct {
    fn derive_redis_value(&self, ident: &Ident, attrs: &HashMap<String, String>) -> TokenStream {
        let rename_all_opt = attrs.get("rename_all").map(|s| s.as_str());
        let (stringified_idents, idents): (Vec<String>, Vec<&Ident>) =
            match &self.fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .map(|field| {
                        let field_ident = field.ident.as_ref().unwrap();
                        let attrs = parse::attributes(&field.attrs);
                        let stringified_ident = attrs
                            .get("rename")
                            .map(ToOwned::to_owned)
                            .unwrap_or_else(|| {
                                string::transform_casing(&field_ident.to_string(), rename_all_opt)
                            });

                        (stringified_ident, field_ident)
                    })
                    .unzip(),
                Fields::Unnamed(fields_unnamed) => {
                    let (indices, types): (Vec<_>, Vec<_>) = (0..fields_unnamed.unnamed.len())
                        .map(|i| {
                            let ident = fields_unnamed.unnamed[i].ident.as_ref().unwrap();
                            (i.to_string(), ident)
                        })
                        .unzip();
                    (indices, types)
                }
                Fields::Unit => return quote! {
                    impl ::redis_om::redis::ToRedisArgs for #ident {
                        fn write_redis_args<W : ?Sized + ::redis_om::redis::RedisWrite>(&self, out: &mut W) {}
                    }

                    impl ::redis_om::redis::FromRedisValue for #ident {
                        fn from_redis_value(_: &::redis_om::redis::Value) -> ::redis_om::redis::RedisResult<Self> {
                            Ok(Self{})
                        }
                    }

                }
                .into(),
            };

        let err_msg = "the data returned is not in the bulk data format or the length is not devisable by two";
        let err = quote!(RedisError::from((
            ErrorKind::TypeError,
            #err_msg,
            format!("{:#?}", v)
        )));

        quote! {
            impl ::redis_om::redis::ToRedisArgs for #ident {
                fn write_redis_args<W : ?Sized + ::redis_om::redis::RedisWrite>(&self, out: &mut W) {
                    use ::redis_om::redis::*;
                    #(
                        match ToRedisArgs::to_redis_args(&self.#idents) {
                            redis_args if redis_args.len() == 1 => {
                                out.write_arg_fmt(#stringified_idents);
                                out.write_arg(&redis_args[0]);
                            },
                            redis_args => {
                                for (idx, item) in redis_args.iter().enumerate() {
                                    out.write_arg_fmt(format!("{}.{}", #stringified_idents, idx));
                                    out.write_arg(&item)
                                }
                            }
                        }
                     )*
                }
            }


            impl ::redis_om::redis::FromRedisValue for #ident {
                fn from_redis_value(v: &::redis_om::redis::Value) -> ::redis_om::redis::RedisResult<Self> {
                    use ::redis_om::redis::*;

                    let Value::Bulk(bulk) = v else { return Err(#err); };

                    if bulk.len() % 2 != 0 { return Err(#err); };

                    let mut fm = std::collections::HashMap::new();

                    for chunks in bulk.chunks(2) {
                        let key: String = from_redis_value(&chunks[0])?;
                        let value: Value = chunks[1].clone();

                        let Some((key, idx)) = key.split_once(".") else { fm.insert(key, value); continue; };
                        let Some(Value::Bulk(vec)) = fm.get_mut(key) else { fm.insert(key.into(), Value::Bulk(vec![value])); continue; };

                        vec.push(value);
                    }

                    Ok(Self {
                        #(#idents: from_redis_value(fm.get(#stringified_idents).unwrap_or(&Value::Nil))?,)*
                    })
                }
            }
        }
        .into()
    }
}
