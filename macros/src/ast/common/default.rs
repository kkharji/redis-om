/// Represents the default to use for a field when deserializing.
pub enum Default {
    /// Field must always be specified because it does not have a default.
    None,
    /// The default is given by `std::default::Default::default()`.
    Default,
    /// The default is given by this function.
    Path(syn::ExprPath),
}

impl Default {
    pub fn is_none(&self) -> bool {
        match self {
            Default::None => true,
            Default::Default | Default::Path(_) => false,
        }
    }
}
