mod attr;

use crate::ast::Ctx;
pub use attr::ContainerAttr;
use syn::{punctuated::Punctuated, Token};

use super::{Data, Default, Field, FieldAttr, Style, Variant, VariantAttr};

pub struct Container<'a> {
    /// The struct or enum name (without generics).
    pub ident: &'a syn::Ident,
    /// The contents of the struct or enum.
    pub data: Data<'a>,
    /// Attributes on the structure.
    pub attrs: ContainerAttr,
    /// Any generics on the struct or enum.
    pub generics: &'a syn::Generics,
    /// Original input.
    pub original: &'a syn::DeriveInput,
}

impl<'a> Container<'a> {
    /// Convert the raw Syn ast into a parsed container object, collecting errors in `ctx`.
    pub fn new(ctx: &Ctx, item: &'a syn::DeriveInput) -> Option<Container<'a>> {
        let mut attrs = ContainerAttr::from_ast(ctx, item);

        let mut data = match &item.data {
            syn::Data::Enum(data) => Data::Enum(enum_from_ast(ctx, &data.variants, &attrs.default)),
            syn::Data::Struct(data) => {
                let (style, fields) = struct_from_ast(ctx, &data.fields, None, &attrs.default);
                Data::Struct(style, fields)
            }
            syn::Data::Union(_) => {
                ctx.error_spanned_by(item, "Serde does not support derive for unions");
                return None;
            }
        };

        let mut has_flatten = false;
        match &mut data {
            Data::Enum(variants) => {
                for variant in variants {
                    variant.attrs.rename_by_rules(&attrs.rename_all_rules);
                    for field in &mut variant.fields {
                        if field.attrs.flatten() {
                            has_flatten = true;
                        }
                        field.attrs.rename_by_rules(&variant.attrs.rename_all_rules);
                    }
                }
            }
            Data::Struct(_, fields) => {
                for field in fields {
                    if field.attrs.flatten() {
                        has_flatten = true;
                    }
                    field.attrs.rename_by_rules(&attrs.rename_all_rules);
                }
            }
        }

        if has_flatten {
            attrs.mark_has_flatten();
        }

        let item = Container {
            ident: &item.ident,
            attrs,
            data,
            generics: &item.generics,
            original: item,
        };

        Some(item)
    }
}

fn enum_from_ast<'a>(
    cx: &Ctx,
    variants: &'a Punctuated<syn::Variant, Token![,]>,
    container_default: &Default,
) -> Vec<Variant<'a>> {
    variants
        .iter()
        .map(|variant| {
            let attrs = VariantAttr::from_ast(cx, variant);
            let (style, fields) =
                struct_from_ast(cx, &variant.fields, Some(&attrs), container_default);
            Variant {
                ident: variant.ident.clone(),
                attrs,
                style,
                fields,
                inner: variant,
            }
        })
        .collect()
}

fn struct_from_ast<'a>(
    cx: &Ctx,
    fields: &'a syn::Fields,
    attrs: Option<&VariantAttr>,
    container_default: &Default,
) -> (Style, Vec<Field<'a>>) {
    match fields {
        syn::Fields::Named(fields) => (
            Style::Struct,
            fields_from_ast(cx, &fields.named, attrs, container_default),
        ),
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => (
            Style::Newtype,
            fields_from_ast(cx, &fields.unnamed, attrs, container_default),
        ),
        syn::Fields::Unnamed(fields) => (
            Style::Tuple,
            fields_from_ast(cx, &fields.unnamed, attrs, container_default),
        ),
        syn::Fields::Unit => (Style::Unit, Vec::new()),
    }
}

fn fields_from_ast<'a>(
    cx: &Ctx,
    fields: &'a Punctuated<syn::Field, Token![,]>,
    attrs: Option<&VariantAttr>,
    container_default: &Default,
) -> Vec<Field<'a>> {
    let mut fields = fields
        .iter()
        .enumerate()
        .map(|(i, field)| Field {
            ident: field.ident.as_ref(),
            member: match &field.ident {
                Some(ident) => syn::Member::Named(ident.clone()),
                None => syn::Member::Unnamed(i.into()),
            },
            attrs: FieldAttr::from_ast(cx, i, field, attrs, container_default),
            ty: &field.ty,
            original: field,
        })
        .collect::<Vec<Field<'a>>>();

    let pk_is_set = fields.iter().any(|f| f.attrs.primary_key);
    if !pk_is_set {
        if let Some(id_field) = fields
            .iter_mut()
            .find(|f| f.attrs.name.serialize_name() == "id")
        {
            id_field.attrs.primary_key = true;
        } else {
            let msg = format!("A primary field doesn't exists, either add `id` field or annotate a field `primary_key`");
            cx.error_spanned_by("", msg);
        };
    }
    fields
}
