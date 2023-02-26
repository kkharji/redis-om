use super::TokenStream;
use crate::ast::Variant;
use crate::ast::{Container, Ctx, Style};
use crate::util::{parse::AttributeMap, string};
use quote::quote;
use syn::{DataEnum, Fields, Ident};

pub(super) fn derive(ctx: &Ctx, cont: &Container, variants: &[Variant]) -> Result<TokenStream, ()> {
    if !variants.iter().all(|v| {
        let is_unit = matches!(v.style, Style::Unit);
        if !is_unit {
            ctx.error_spanned_by(v.inner, "Only Enum's Unit variant is currently supported");
        };
        is_unit
    }) {
        return Err(());
    };

    let to_redis_args = derive_to_redis_args(ctx, cont, variants)?;
    let from_redis_args = derive_from_redis_args(ctx, cont, variants)?;

    Ok(quote![
       #to_redis_args
       #from_redis_args
    ])
}

fn derive_to_redis_args(
    ctx: &Ctx,
    cont: &Container,
    variants: &[Variant],
) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let matches = variants
        .iter()
        .filter(|v| !v.attrs.skip_serializing)
        .map(|v| {
            let name = &v.ident;
            let value = v.attrs.name.serialize_name();
            quote!(#type_name::#name => out.write_arg(#value.as_bytes()),)
        });

    Ok(quote! {
        impl ::redis_om::redis::ToRedisArgs for #type_name {
            fn write_redis_args<W: ?Sized + ::redis_om::redis::RedisWrite>(&self, out: &mut W) {
                match self { #(#matches)* }
            }
        }
    })
}

fn derive_from_redis_args(
    ctx: &Ctx,
    cont: &Container,
    variants: &[Variant],
) -> Result<TokenStream, ()> {
    let type_name = cont.ident;
    let except_redis_string = quote! {
        let msg = format!("{:?}", v);
        return Err((TypeError, "Expected Redis string, got:", msg).into());
    };

    let (values, matches): (Vec<_>, Vec<_>) = variants
        .iter()
        .filter(|v| !v.attrs.skip_deserializing)
        .map(|v| {
            let name = &v.ident;
            let values = v.attrs.name.deserialize_aliases();
            let matches: Vec<_> = values
                .iter()
                .map(|value| quote! { #value => Ok(#type_name::#name), })
                .collect();
            (values, matches)
        })
        .unzip();

    let values = values.into_iter().flatten().collect::<Vec<_>>().join(", ");
    let matches = matches.into_iter().flatten();

    let except_specifc_values = quote! {{
        let msg = format!("{}, Expected one of: {}", v, #values);
        Err((TypeError, "Invalid enum variant:", msg).into())
    }};

    Ok(quote! {
        impl ::redis_om::redis::FromRedisValue for #type_name {
            fn from_redis_value(v: &::redis_om::redis::Value) -> ::redis_om::RedisResult<Self> {
                use ::redis_om::redis::{ErrorKind::TypeError, Value};

                let redis::Value::Data(data) = v else { #except_redis_string };
                let value = std::str::from_utf8(&data[..])?;

                match value {
                    #(#matches)*
                    v => #except_specifc_values,
                }
            }
        }
    })
}
