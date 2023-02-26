use quote::ToTokens;
use syn::{ext::IdentExt, Meta, NestedMeta};
use Meta::*;
use NestedMeta::*;

use crate::{
    ast::{symbols::*, Attr, BoolAttr, Ctx, Default, Name, RenameAllRules, RenameRule, VecAttr},
    ext::{LitExt, MetaListExt},
};

/// Represents struct or enum attributes supported by redis-om.
pub struct VariantAttr {
    pub name: Name,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    pub rename_all_rules: RenameAllRules,
}

impl VariantAttr {
    /// Extract out the `#[redis(...)]` attributes from an item.
    pub(crate) fn from_ast(ctx: &Ctx, variant: &syn::Variant) -> Self {
        let mut default: Attr<Default> = Attr::new(ctx, DEFUALT);
        let mut ser_name = Attr::new(ctx, RENAME);
        let mut de_name = Attr::new(ctx, RENAME);
        let mut de_aliases = VecAttr::new(ctx, RENAME);
        let mut skip_deserializing = BoolAttr::new(ctx, SKIP_DESERIALIZING);
        let mut skip_serializing = BoolAttr::new(ctx, SKIP_SERIALIZING);
        let mut rename_all_ser_rule = Attr::new(ctx, RENAME_ALL);
        let mut rename_all_de_rule = Attr::new(ctx, RENAME_ALL);

        variant
            .attrs
            .iter()
            .flat_map(|attr| {
                if attr.path != SYMBOL_REDIS {
                    return Ok(Vec::new());
                }

                match attr.parse_meta() {
                    Ok(Meta::List(meta)) => Ok(meta.nested.into_iter().collect()),
                    Ok(other) => {
                        ctx.error_spanned_by(other, "expected #[redis(...)]");
                        Err(())
                    }
                    Err(err) => {
                        ctx.syn_error(err);
                        Err(())
                    }
                }
            })
            .flatten()
            .for_each(|item| match &item {
                // Parse `#[redis(rename = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME => {
                    if let Ok(s) = m.lit.to_lit_str(ctx, RENAME) {
                        ser_name.set(&m.path, s.value());
                        de_name.set_if_none(s.value());
                        de_aliases.insert(&m.path, s.value());
                    }
                }

                // Parse `#[redis(rename(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME => {
                    if let Ok((ser, de)) = m.get_multiple_renames(ctx) {
                        ser_name.set_opt(&m.path, ser.map(syn::LitStr::value));
                        for de_value in de {
                            de_name.set_if_none(de_value.value());
                            de_aliases.insert(&m.path, de_value.value());
                        }
                    }
                }

                // Parse `#[redis(alias = "foo")]`
                Meta(NameValue(m)) if m.path == ALIAS => {
                    if let Ok(s) = m.lit.to_lit_str(ctx, ALIAS) {
                        de_aliases.insert(&m.path, s.value());
                    }
                }

                // Parse `#[redis(default)]` or `#[redis(default = "...")]`
                Meta(Path(key)) if default.eq(key) => {
                    default.set(key, Default::Default);
                }
                Meta(NameValue(m)) if default.eq(&m.path) => {
                    if let Ok(path) = m.lit.to_expr_path(ctx, DEFUALT) {
                        default.set(&m.path, Default::Path(path));
                    }
                }

                // Parse `#[redis(skip)]`
                Meta(Path(word)) if word == SKIP => {
                    skip_serializing.set_true(word);
                    skip_deserializing.set_true(word);
                }

                // Parse `#[redis(skip_deserializing)]`
                Meta(Path(word)) if word == SKIP_DESERIALIZING => {
                    skip_deserializing.set_true(word);
                }

                // Parse `#[redis(skip_serializing)]`
                Meta(Path(word)) if word == SKIP_SERIALIZING => {
                    skip_serializing.set_true(word);
                }

                // Parse `#[redis(rename_all = "foo")]`
                Meta(NameValue(m)) if m.path == RENAME_ALL => {
                    if let Ok(s) = m.lit.to_lit_str(ctx, RENAME_ALL) {
                        match RenameRule::from_str(&s.value()) {
                            Ok(rename_rule) => {
                                rename_all_ser_rule.set(&m.path, rename_rule);
                                rename_all_de_rule.set(&m.path, rename_rule);
                            }
                            Err(err) => ctx.error_spanned_by(s, err),
                        }
                    }
                }

                // Parse `#[redis(rename_all(serialize = "foo", deserialize = "bar"))]`
                Meta(List(m)) if m.path == RENAME_ALL => {
                    if let Ok((ser, de)) = m.get_renames(ctx) {
                        if let Some(ser) = ser {
                            match RenameRule::from_str(&ser.value()) {
                                Ok(rename_rule) => rename_all_ser_rule.set(&m.path, rename_rule),
                                Err(err) => ctx.error_spanned_by(ser, err),
                            }
                        }
                        if let Some(de) = de {
                            match RenameRule::from_str(&de.value()) {
                                Ok(rename_rule) => rename_all_de_rule.set(&m.path, rename_rule),
                                Err(err) => ctx.error_spanned_by(de, err),
                            }
                        }
                    }
                }

                Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");

                    let msg = format!("unknown redis field attribute `{path}`");
                    ctx.error_spanned_by(meta_item.path(), msg);
                }

                Lit(lit) => {
                    let msg = "unexpected literal in redis container attribute";
                    ctx.error_spanned_by(lit, msg);
                }
            });

        Self {
            name: Name::from_attrs(
                variant.ident.unraw().to_string(),
                ser_name,
                de_name,
                Some(de_aliases),
            ),
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
        }
    }

    pub fn rename_by_rules(&mut self, rules: &RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize = rules.serialize.apply_to_variant(&self.name.serialize);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize = rules.deserialize.apply_to_variant(&self.name.deserialize);
        }
    }
}
