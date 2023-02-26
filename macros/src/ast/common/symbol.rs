use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(pub &'static str);

pub mod symbols {
    use super::*;
    pub const SYMBOL_REDIS: Symbol = Symbol("redis");
    pub const DESERIALIZE: Symbol = Symbol("deserialize");
    pub const SERIALIZE: Symbol = Symbol("serialize");
    pub const DEFUALT: Symbol = Symbol("default");
    pub const PRIMARY_KEY: Symbol = Symbol("primary_key");
    pub const PREFIX_KEY: Symbol = Symbol("prefix_key");
    pub const MODEL_TYPE: Symbol = Symbol("model_type");
    pub const INDEX: Symbol = Symbol("index");
    pub const SORTABLE: Symbol = Symbol("sortable");
    pub const FULL_TEXT_SEARCH: Symbol = Symbol("full_text_search");
    pub const RENAME: Symbol = Symbol("rename");
    pub const RENAME_ALL: Symbol = Symbol("rename_all");
    pub const ALIAS: Symbol = Symbol("alias");
    pub const FLATTEN: Symbol = Symbol("flatten");
    pub const SKIP: Symbol = Symbol("skip_serializing");
    pub const SKIP_SERIALIZING: Symbol = Symbol("skip_serializing");
    pub const SKIP_DESERIALIZING: Symbol = Symbol("skip_deserializing");
}

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
