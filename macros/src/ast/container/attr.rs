use crate::{
    ast::{symbols::*, *},
    derive::Derive,
    ext::{LitExt, MetaListExt},
};
use quote::ToTokens;
use syn::{Data::*, DataEnum, DataStruct, DataUnion, Meta, NestedMeta};
use Meta::*;
use NestedMeta::*;

/// Represents struct or enum attributes supported by redis-om.
pub struct ContainerAttr {
    /// Redis database prefix key or redis stream name
    pub prefix_key: String,
    /// Which key is considered primary key
    pub primary_key: String,
    /// Default alternative for missing fields
    pub default: Default,
    /// Rename all rules
    pub rename_all_rules: RenameAllRules,
    /// Rename all rules
    pub model_type: Derive,
    has_flatten: bool,
}

impl ContainerAttr {
    /// Extract out the `#[redis(...)]` attributes from an item.
    pub(crate) fn from_ast(ctx: &Ctx, input: &syn::DeriveInput) -> Self {
        let mut default: Attr<Default> = Attr::new(ctx, DEFUALT);
        let mut primary_key: Attr<String> = Attr::new(ctx, PRIMARY_KEY);
        let mut prefix_key: Attr<String> = Attr::new(ctx, PREFIX_KEY);
        let mut model_type: Attr<Derive> = Attr::new(ctx, MODEL_TYPE);
        let mut rename_all_ser_rule = Attr::new(ctx, RENAME_ALL);
        let mut rename_all_de_rule = Attr::new(ctx, RENAME_ALL);

        input
            .attrs
            .iter()
            .flat_map(|attr| {
                if attr.path != symbols::SYMBOL_REDIS {
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
                // Parse `#[redis(default)]`
                Meta(Path(word)) if default.eq(word) => match &input.data {
                    Struct(DataStruct { fields: syn::Fields::Named(_), .. }) => default.set(word, Default::Default),
                    Struct(DataStruct { fields, .. }) => ctx.error_spanned_by(
                        fields,
                        "#[redis(default)] can only be used on structs with named fields",
                    ),
                    Enum(DataEnum { enum_token, .. }) => ctx.error_spanned_by(
                        enum_token,
                        "#[redis(default)] can only be used on structs with named fields",
                    ),
                    Union(DataUnion { union_token, .. }) => ctx.error_spanned_by(
                        union_token,
                        "#[redis(default)] can only be used on structs with named fields",
                    ),
                },

                // Parse `#[redis(default = "...")]`
                Meta(NameValue(nv)) if default.eq(&nv.path) => {
                    if let Ok(path) = nv.lit.to_expr_path(ctx, DEFUALT) {
                        match &input.data {
                            Struct(DataStruct { fields: syn::Fields::Named(_), .. }) => {
                                default.set(&nv.path, Default::Path(path));
                            }

                            Struct(DataStruct { fields, .. }) => {
                                let msg = &"#[redis(default = \"...\")] can only be used on structs with named fields";
                                ctx.error_spanned_by(fields, msg)
                            }

                            Enum(DataEnum { enum_token, .. }) => {
                                let msg = &"#[redis(default = \"...\")] can only be used on structs with named fields";
                                ctx.error_spanned_by( enum_token, msg)
                            },

                            Union(DataUnion {
                                union_token, ..
                            }) => {
                                let msg = &"#[redis(default = \"...\")] can only be used on structs with named fields";
                                ctx.error_spanned_by( union_token, msg)
                            },
                        }
                    }
                }

                // Parse `#[redis(prefix_key = "...")]`
                Meta(NameValue(nv)) if prefix_key.eq(&nv.path) => {
                    prefix_key.set_opt(&nv.path, nv.lit.to_string());
                }

                // Parse `#[redis(stream_key = "...")]`
                Meta(NameValue(nv)) if &nv.path == KEY => {
                    prefix_key.set_opt(&nv.path, nv.lit.to_string());
                }

                // Parse `#[redis(primary_key = "...")]`
                Meta(NameValue(nv)) if primary_key.eq(&nv.path) => {
                    primary_key.set_opt(&nv.path, nv.lit.to_string());
                }

                // Parse `#[redis(model_type = "...")]`
                Meta(NameValue(nv)) if &nv.path == MODEL_TYPE => {
                    let value = nv.lit.to_lit_str(ctx, MODEL_TYPE).and_then(|s| Derive::from_lit_str(ctx, s));
                    model_type.set_opt(&nv.path, value.ok());
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


                Lit(lit) => {
                    let msg = "unexpected literal in redis container attribute";
                    ctx.error_spanned_by(lit, msg);
                }

               Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");
                    ctx.error_spanned_by(
                        meta_item.path(),
                        format!("unknown redis container attribute `{}`", path),
                    );
                }
            });

        Self {
            primary_key: primary_key.get().unwrap_or_else(|| "id".into()),
            prefix_key: prefix_key.get().unwrap_or_else(|| input.ident.to_string()),
            default: default.get().unwrap_or(Default::None),
            has_flatten: false,
            model_type: model_type.get().unwrap_or(Derive::HashModel),
            rename_all_rules: RenameAllRules {
                serialize: rename_all_ser_rule.get().unwrap_or(RenameRule::None),
                deserialize: rename_all_de_rule.get().unwrap_or(RenameRule::None),
            },
        }
    }

    pub fn mark_has_flatten(&mut self) {
        self.has_flatten = true;
    }
}
