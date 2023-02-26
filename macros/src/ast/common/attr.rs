use crate::ast::{Ctx, Symbol};
use proc_macro2::TokenStream;
use quote::ToTokens;

pub struct Attr<'c, T> {
    cx: &'c Ctx,
    name: Symbol,
    tokens: TokenStream,
    value: Option<T>,
}

impl<'c, T> Attr<'c, T> {
    pub fn new(cx: &'c Ctx, name: Symbol) -> Self {
        Attr {
            cx,
            name,
            tokens: TokenStream::new(),
            value: None,
        }
    }

    pub fn set<A: ToTokens>(&mut self, obj: A, value: T) {
        let tokens = obj.into_token_stream();

        if self.value.is_some() {
            self.cx
                .error_spanned_by(tokens, format!("duplicate redis attribute `{}`", self.name));
        } else {
            self.tokens = tokens;
            self.value = Some(value);
        }
    }

    pub fn set_opt<A: ToTokens>(&mut self, obj: A, value: Option<T>) {
        if let Some(value) = value {
            self.set(obj, value);
        }
    }

    pub fn set_if_none(&mut self, value: T) {
        if self.value.is_none() {
            self.value = Some(value);
        }
    }

    pub fn get(self) -> Option<T> {
        self.value
    }

    pub fn eq<V: PartialEq<Symbol>>(&self, name: &V) -> bool {
        name.eq(&self.name)
    }

    pub fn get_with_tokens(self) -> Option<(TokenStream, T)> {
        match self.value {
            Some(v) => Some((self.tokens, v)),
            None => None,
        }
    }
}

pub struct BoolAttr<'c>(Attr<'c, ()>);

impl<'c> BoolAttr<'c> {
    pub fn new(cx: &'c Ctx, name: Symbol) -> Self {
        BoolAttr(Attr::new(cx, name))
    }

    pub fn set_true<A: ToTokens>(&mut self, obj: A) {
        self.0.set(obj, ());
    }

    pub fn get(&self) -> bool {
        self.0.value.is_some()
    }

    pub fn eq<V: PartialEq<Symbol>>(&self, name: &V) -> bool {
        self.0.eq(name)
    }
}

pub struct VecAttr<'c, T> {
    cx: &'c Ctx,
    name: Symbol,
    first_dup_tokens: TokenStream,
    values: Vec<T>,
}

impl<'c, T> VecAttr<'c, T> {
    pub fn new(cx: &'c Ctx, name: Symbol) -> Self {
        VecAttr {
            cx,
            name,
            first_dup_tokens: TokenStream::new(),
            values: Vec::new(),
        }
    }

    pub fn insert<A: ToTokens>(&mut self, obj: A, value: T) {
        if self.values.len() == 1 {
            self.first_dup_tokens = obj.into_token_stream();
        }
        self.values.push(value);
    }

    pub fn at_most_one(mut self) -> Result<Option<T>, ()> {
        if self.values.len() > 1 {
            let dup_token = self.first_dup_tokens;
            self.cx.error_spanned_by(
                dup_token,
                format!("duplicate neo4j attribute `{}`", self.name),
            );
            Err(())
        } else {
            Ok(self.values.pop())
        }
    }

    pub fn get(self) -> Vec<T> {
        self.values
    }
}
