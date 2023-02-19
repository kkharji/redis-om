use proc_macro2::TokenStream;
use syn::parse::Parser;
use syn::Attribute;

pub trait AttributeExt {
    type Target: Sized;

    fn from_token_stream(_: TokenStream) -> syn::Result<Self::Target>;

    fn from_str(_: &str) -> syn::Result<Self::Target>;
}

impl AttributeExt for Attribute {
    type Target = Self;

    fn from_token_stream(input: TokenStream) -> syn::Result<Self::Target> {
        let parser = Self::parse_outer;

        Ok(parser.parse2(input)?.into_iter().next().unwrap())
    }

    fn from_str(input: &str) -> syn::Result<Self::Target> {
        let parser = Self::parse_outer;

        Ok(parser.parse_str(input)?.into_iter().next().unwrap())
    }
}
