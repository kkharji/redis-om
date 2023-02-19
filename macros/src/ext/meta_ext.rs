use super::PathExt;
use syn::Meta;

pub trait MetaExt {
    fn to_string(&self) -> String;
}

impl MetaExt for Meta {
    fn to_string(&self) -> String {
        self.path().to_string()
    }
}
