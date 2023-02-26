use quote::ToTokens;
use syn::{Meta, NestedMeta};
use Meta::*;
use NestedMeta::*;

use crate::{
    ast::{symbols::*, Attr, BoolAttr, Ctx, Default, Name, RenameAllRules, VariantAttr, VecAttr},
    ext::{LitExt, MetaListExt},
};

/// Represents struct or enum attributes supported by redis-om.
pub struct FieldAttr {
    pub name: Name,
    /// Default alternative for missing fields
    pub default: Default,
    /// Whether the key should be considered a primary key
    pub primary_key: bool,
    /// Whether the key should be indexed
    pub index: bool,
    /// Whether the key should be sortable
    pub sortable: bool,
    /// Whether the key should be full text search
    pub fts: bool,
    pub skip_deserializing: bool,
    pub skip_serializing: bool,
    flatten: bool,
}

impl FieldAttr {
    /// Extract out the `#[redis(...)]` attributes from an item.
    pub(crate) fn from_ast(
        ctx: &Ctx,
        index: usize,
        field: &syn::Field,
        _attrs: Option<&VariantAttr>,
        _container_default: &Default,
    ) -> Self {
        let ident = match &field.ident {
            Some(ident) => ident.to_string().trim_start_matches("r#").to_owned(),
            None => index.to_string(),
        };

        let mut default: Attr<Default> = Attr::new(ctx, DEFUALT);
        let mut ser_name = Attr::new(ctx, RENAME);
        let mut de_name = Attr::new(ctx, RENAME);
        let mut de_aliases = VecAttr::new(ctx, RENAME);
        let mut flatten = BoolAttr::new(ctx, FLATTEN);
        let mut skip_deserializing = BoolAttr::new(ctx, SKIP_DESERIALIZING);
        let mut skip_serializing = BoolAttr::new(ctx, SKIP_SERIALIZING);

        let mut primary_key: BoolAttr = BoolAttr::new(ctx, PRIMARY_KEY);
        let mut index: BoolAttr = BoolAttr::new(ctx, INDEX);
        let mut sortable: BoolAttr = BoolAttr::new(ctx, SORTABLE);
        let mut fts: BoolAttr = BoolAttr::new(ctx, FULL_TEXT_SEARCH);

        field
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

                // Parse `#[redis(primary_key)]`
                Meta(Path(key)) if primary_key.eq(&key) => primary_key.set_true(key),
                Meta(NameValue(nv)) if primary_key.eq(&nv.path) => {
                    let msg = "unexpected value for primary_key, use just #[redis(primary_key)]";
                    ctx.error_spanned_by(&nv.path, msg);
                }

                // Parse `#[redis(index)]`
                Meta(Path(key)) if index.eq(&key) => index.set_true(key),
                Meta(NameValue(nv)) if index.eq(&nv.path) => {
                    let msg = "unexpected value for index, use #[redis(index)]";
                    ctx.error_spanned_by(&nv.path, msg);
                }

                // Parse `#[redis(sortable)]`
                Meta(Path(key)) if sortable.eq(&key) => sortable.set_true(key),
                Meta(NameValue(nv)) if sortable.eq(&nv.path) => {
                    let msg = "unexpected value for sortable, use #[redis(sortable)]";
                    ctx.error_spanned_by(&nv.path, msg);
                }

                // Parse `#[redis(full_text_search)]`
                Meta(Path(key)) if fts.eq(&key) => fts.set_true(key),
                Meta(NameValue(nv)) if fts.eq(&nv.path) => {
                    let msg =
                        "unexpected value for full_text_search, use #[redis(full_text_search)]";
                    ctx.error_spanned_by(&nv.path, msg);
                }

                // Parse `#[redis(flatten)]`
                Meta(Path(word)) if word == FLATTEN => {
                    flatten.set_true(word);
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
            name: Name::from_attrs(ident, ser_name, de_name, Some(de_aliases)),
            default: default.get().unwrap_or(Default::None),
            primary_key: primary_key.get(),
            index: index.get(),
            sortable: sortable.get(),
            fts: fts.get(),
            flatten: flatten.get(),
            skip_serializing: skip_serializing.get(),
            skip_deserializing: skip_deserializing.get(),
        }
    }

    pub fn rename_by_rules(&mut self, rules: &RenameAllRules) {
        if !self.name.serialize_renamed {
            self.name.serialize = rules.serialize.apply_to_field(&self.name.serialize);
        }
        if !self.name.deserialize_renamed {
            self.name.deserialize = rules.deserialize.apply_to_field(&self.name.deserialize);
        }
    }

    pub fn flatten(&self) -> bool {
        self.flatten
    }
}
