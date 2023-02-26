use super::{Field, Style, Variant};

/// The fields of a struct or enum.
///
/// Analogous to `syn::Data`.
pub enum Data<'a> {
    Enum(Vec<Variant<'a>>),
    Struct(Style, Vec<Field<'a>>),
}

impl<'a> Data<'a> {
    pub fn fields(&'a self) -> Box<dyn Iterator<Item = &'a Field<'a>> + 'a> {
        match self {
            Data::Enum(variants) => Box::new(variants.iter().flat_map(|v| v.fields.iter())),
            Data::Struct(_, fields) => Box::new(fields.iter()),
        }
    }
}
