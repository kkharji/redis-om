use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericArgument, Ident, Path, PathArguments, Type};

pub trait TypeExt {
    fn path(&self) -> Option<&Path>;

    fn arguments(&self) -> Option<Vec<TokenStream>>;

    fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>;

    fn is_primitive_copy(&self) -> bool;

    fn is_option(&self) -> bool;

    fn to_as_ref(&self) -> Option<TokenStream>;

    fn is_reference(&self) -> bool;

    fn is_list_collection(&self) -> bool;

    fn is_numeric_type(&self) -> bool;

    fn get_inner_type(&self) -> Option<&Type>;
}

impl TypeExt for Type {
    fn is_ident<I: ?Sized>(&self, ident: &I) -> bool
    where
        Ident: PartialEq<I>,
    {
        let mut ty = self;
        if self.is_option() {
            ty = self.get_inner_type().unwrap();
        }
        let Some(path) = ty.path() else { return false; };
        let Some(last) = path.segments.last() else {return false;};

        last.ident == *ident
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
        matches!(self, Self::Reference(_))
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
        let Some(path) = self.path() else { return  false};
        let Some(last) = path.segments.last() else {return false;};

        if last.ident != "Option" {
            return false;
        }

        if let PathArguments::AngleBracketed(bracketed) = &last.arguments {
            return bracketed.args.len() == 1;
        }

        false
    }

    fn to_as_ref(&self) -> Option<TokenStream> {
        let Some(path) = self.path() else { return None; };
        let Some(last) = path.segments.last() else { return None; };

        if last.ident == "Option" {
            if let PathArguments::AngleBracketed(bracketed) = &last.arguments {
                let args = &bracketed.args;
                let ident = &last.ident;

                return Some(quote!(#ident<&#args>));
            }
        }

        None
    }

    fn is_list_collection(&self) -> bool {
        let Some(path) = self.path() else { return false };
        let Some(last) = path.segments.last() else { return false; };

        pub(crate) const LIST_COLLECTION_TYPES: [&str; 6] = [
            "Vec",
            "LinkedList",
            "VecDeque",
            "BinaryHeap",
            "HashSet",
            "BTreeSet",
        ];

        if LIST_COLLECTION_TYPES.contains(&last.ident.to_string().as_str()) {
            if let PathArguments::AngleBracketed(bracketed) = &last.arguments {
                return bracketed.args.len() == 1;
            }
        }

        false
    }

    fn is_numeric_type(&self) -> bool {
        let Some(path) = self.path() else {return false;};
        let Some(ident) = path.segments.last().map(|v| v.ident.to_string()) else { return false; };

        matches!(
            ident.as_str(),
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
        )
    }

    fn get_inner_type(&self) -> Option<&syn::Type> {
        let Some(path) = self.path() else { return None; };

        if let PathArguments::AngleBracketed(ref a) = path.segments.first()?.arguments {
            if let syn::GenericArgument::Type(ty) = a.args.first()? {
                return Some(ty);
            }
        };

        None
    }
}

#[test]
fn test_is_list_collection() {
    let ty: syn::Type = syn::parse_str("Vec<String>").unwrap();
    assert_eq!(ty.is_list_collection(), true);
    let ty: syn::Type = syn::parse_str("HashSet<u32>").unwrap();
    assert_eq!(ty.is_list_collection(), true);
}

#[test]
fn test_is_numeric() {
    let ty: syn::Type = syn::parse_str("i32").unwrap();
    assert_eq!(ty.is_numeric_type(), true);
    let ty: syn::Type = syn::parse_str("usize").unwrap();
    assert_eq!(ty.is_numeric_type(), true);
}

#[test]
fn test_get_inner_type() {
    let ty = syn::parse_str::<Type>("Vec<u32>").unwrap();
    let inner = ty.get_inner_type();
    assert!(inner.map(|v| v.is_ident("u32")).unwrap());
    let ty = syn::parse_str::<Type>("HashSet<String>").unwrap();
    let inner = ty.get_inner_type();
    assert!(inner.map(|v| v.is_ident("String")).unwrap());
}
