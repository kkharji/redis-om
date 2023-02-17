use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::{Attribute, GenericArgument, Ident, Path, PathArguments, Type};

pub mod parse;
pub mod string;

pub(crate) trait AttributeExt {
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

pub(crate) trait TypeExt {
    fn path(&self) -> Option<&Path>;

    fn arguments(&self) -> Option<Vec<TokenStream>>;

    fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>;

    fn is_primitive_copy(&self) -> bool;

    fn is_option(&self) -> bool;

    fn to_as_ref(&self) -> Option<TokenStream>;

    fn is_reference(&self) -> bool;
}

impl TypeExt for Type {
    fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>,
    {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            last.ident == *ident
        } else {
            false
        }
    }

    fn path(&self) -> Option<&Path> {
        match &self {
            Self::Path(ty) => Some(&ty.path),
            _ => None,
        }
    }

    fn arguments(&self) -> Option<Vec<TokenStream>> {
        let path = self.path()?;

        if let Some(last) = path.segments.last() {
            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => {
                    return Some(bracketed.args.iter().map(|v| quote!(#v)).collect())
                }
                _ => return None,
            }
        }

        None
    }

    fn is_reference(&self) -> bool {
        match &self {
            Self::Reference(_) => true,
            _ => false,
        }
    }

    fn is_primitive_copy(&self) -> bool {
        let path = match &self {
            // Array types of all sizes implement copy, if the item type implements copy.
            Self::Array(ty) => return ty.elem.is_primitive_copy(),
            Self::Group(ty) => return ty.elem.is_primitive_copy(),
            Self::Paren(ty) => return ty.elem.is_primitive_copy(),
            Self::Path(ty) => &ty.path,
            // mutable references do not implement copy:
            Self::Reference(ty) => return ty.mutability.is_none(),
            Self::Tuple(ty) => return !ty.elems.iter().map(|s| s.is_primitive_copy()).any(|e| !e),
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            match last.ident.to_string().as_ref() {
                "bool" | "char" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128"
                | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => return true,
                "Option" => {
                    if let PathArguments::AngleBracketed(bracketed) = &last.arguments {
                        let mut result = true;

                        for arg in bracketed.args.iter() {
                            if let GenericArgument::Type(ty) = arg {
                                result = ty.is_primitive_copy();
                            } // all other kinds of GenericArgument are ignored for now...

                            if !result {
                                return result;
                            }
                        }

                        return result;
                    }
                }
                _ => return false,
            }
        }

        false
    }

    fn is_option(&self) -> bool {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return false,
        };

        if let Some(last) = path.segments.last() {
            if last.ident != "Option" {
                return false;
            }

            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => return bracketed.args.len() == 1,
                _ => return false,
            }
        }

        false
    }

    fn to_as_ref(&self) -> Option<TokenStream> {
        let path = match &self {
            Self::Path(ty) => &ty.path,
            _ => return None,
        };

        if let Some(last) = path.segments.last() {
            if last.ident != "Option" {
                return None;
            }

            match &last.arguments {
                PathArguments::AngleBracketed(bracketed) => {
                    let args = &bracketed.args;
                    let ident = &last.ident;
                    return Some(quote!(#ident<&#args>));
                }
                _ => return None,
            }
        }

        None
    }
}
