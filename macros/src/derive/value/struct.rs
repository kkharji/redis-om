use crate::ast::{Container, Ctx, Field, FieldAttr, Style};
use crate::util::{parse, string};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{DataStruct, Fields, Ident};

pub(super) fn derive(
    ctx: &Ctx,
    cont: &Container,
    style: &Style,
    fields: &[Field],
) -> Result<TokenStream, ()> {
    let to_redis_args = derive_to_redis_args(ctx, cont, style, fields)?;
    let from_redis_args = derive_from_redis(ctx, cont, style, fields)?;

    Ok(quote![
       #to_redis_args
       #from_redis_args
    ])
}

fn derive_to_redis_args(
    ctx: &Ctx,
    cont: &Container,
    style: &Style,
    fields: &[Field],
) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let elms = match style {
        Style::Struct => fields
            .iter()
            .filter(|f| !f.attrs.skip_serializing)
            .map(|f| {
                let ident = f.ident.unwrap();
                let key = f.attrs.name.serialize_name();

                quote! {
                    match ToRedisArgs::to_redis_args(&self.#ident) {
                        redis_args if redis_args.len() == 1 => {
                            out.write_arg_fmt(#key);
                            out.write_arg(&redis_args[0]);
                        },
                        redis_args => {
                            for (idx, item) in redis_args.iter().enumerate() {
                                out.write_arg_fmt(format!("{}.{}", #key, idx));
                                out.write_arg(&item)
                            }
                        }
                    }
                }
            }),
        Style::Tuple | Style::Newtype | Style::Unit => {
            let msg = format!("{:?} Struct is not supported", style);
            ctx.error_spanned_by(cont.original, msg);
            return Err(());
        }
    };

    Ok(quote! {
        impl ::redis_om::redis::ToRedisArgs for #type_name {
            fn write_redis_args<W : ?Sized + ::redis_om::redis::RedisWrite>(&self, out: &mut W) {
                use ::redis_om::redis::*;
                #(#elms)*
            }
        }
    })
}

fn derive_from_redis(
    ctx: &Ctx,
    cont: &Container,
    style: &Style,
    fields: &[Field],
) -> Result<TokenStream, ()> {
    let ident = cont.ident;
    match style {
        Style::Struct => {
            let err_msg = "the data is not in the bulk data format or the length is not / 2";
            let err = quote!(
                RedisError::from((ErrorKind::TypeError, #err_msg, format!("{:#?}", v)))
            );
            let (idents, defs): (Vec<_>, Vec<_>) = fields
                .iter()
                .filter(|f| !f.attrs.skip_deserializing)
                .map(|f| {
                    let ident = f.ident.unwrap();
                    let ident_str = ident.to_string();
                    let keys_ident = format_ident!("{}_POSSIBLE_KEYS", ident_str.to_uppercase());
                    let possible_keys = f.attrs.name.deserialize_aliases();
                    let possible_keys_len = possible_keys.len();

                    // TODO: Support default in deserialization
                    let def = quote! {
                        const #keys_ident: [&str; #possible_keys_len] = [#(#possible_keys),*];
                        let #ident = from_redis_value(
                          #keys_ident
                            .into_iter()
                            .find(|v| fm.contains_key(*v))
                            .map(|v| fm.get(v).unwrap())
                            .unwrap_or(&Value::Nil),
                        )?;
                    };
                    (ident, def)
                })
                .unzip();

            Ok(quote! {
                impl ::redis_om::redis::FromRedisValue for #ident {
                    fn from_redis_value(v: &::redis_om::redis::Value) -> ::redis_om::redis::RedisResult<Self> {
                        use ::redis_om::redis::*;

                        let Value::Bulk(bulk) = v else { return Err(#err); };
                        if bulk.len() % 2 != 0 { return Err(#err); };
                        let mut fm = std::collections::HashMap::new();

                        for chunks in bulk.chunks(2) {
                            let key: String = from_redis_value(&chunks[0])?;
                            let value: Value = chunks[1].clone();
                            let Some((key, idx)) = key.split_once(".") else {
                                fm.insert(key, value);
                                continue;
                            };
                            let Some(Value::Bulk(vec)) = fm.get_mut(key) else {
                                fm.insert(key.into(), Value::Bulk(vec![value]));
                                continue;
                            };

                            vec.push(value);
                        }

                        #(#defs)*

                        Ok(Self { #(#idents,)* })
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
