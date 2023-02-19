use syn::Path;

pub trait PathExt {
    fn to_string(&self) -> String;
}

impl PathExt for Path {
    fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<String>>()
            .join("::")
    }
}
