mod attr;
pub use attr::FieldAttr;

/// A field of a struct.
pub struct Field<'a> {
    pub ident: Option<&'a syn::Ident>,
    pub member: syn::Member,
    pub attrs: FieldAttr,
    pub ty: &'a syn::Type,
    pub original: &'a syn::Field,
}
