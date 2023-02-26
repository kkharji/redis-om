use super::{Field, Style};
mod attr;
pub use attr::*;

/// A variant of an enum.
pub struct Variant<'a> {
    pub ident: syn::Ident,
    pub attrs: VariantAttr,
    pub style: Style,
    pub fields: Vec<Field<'a>>,
    pub inner: &'a syn::Variant,
}
