use crate::ext::{AttributeExt, TypeExt};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{punctuated::Punctuated, token::Comma, Field, Ident};

pub(crate) fn get(field_name: &Ident, field_type: &&syn::Type) -> TokenStream {
    let arguments = vec![quote![&self]];
    let method_name = field_name;
    let method_docs = format!("Get reference to `{}`", field_name.to_string());

    let mut attributes = vec![];

    attributes.push(syn::Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());

    let (return_type, body) = (quote![&#field_type], quote![&self.#field_name]);

    quote! {
        #(#attributes)*
        #[doc = #method_docs]
        pub fn #method_name(#(#arguments),*) -> #return_type {
            #body
        }
    }
}

pub fn get_mut(field_name: &Ident, field_type: &&syn::Type) -> TokenStream {
    let arguments = vec![quote![&mut self]];
    let method_name = format_ident!("{}_mut", field_name);
    let method_docs = format!("Get mutable reference to `{}`", field_name.to_string());

    let mut attributes = vec![];

    attributes.push(syn::Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());

    let (return_type, body) = (quote![&mut #field_type], quote![&mut self.#field_name]);

    quote! {
        #(#attributes)*
        #[doc = #method_docs]
        pub fn #method_name(#(#arguments),*) -> #return_type {
            #body
        }
    }
}

pub(crate) fn set(field_name: &Ident, field_type: &&syn::Type) -> TokenStream {
    let mut arguments = vec![quote![&mut self]];
    let method_name = format_ident!("set_{}", field_name);
    let return_type = quote![&mut Self];
    let method_docs = format!("Set `{}` value ", field_name.to_string());

    let mut generics = vec![];
    let mut attributes = vec![];
    let mut argument = quote! { value: VALUE };
    let mut bound = quote! { VALUE: ::std::convert::Into<#field_type> };
    let mut assignment = quote! { self.#field_name = value.into();; };

    if field_type.is_ident("Option") {
        // tries to get the `T` from Option<T>
        if let Some(arg) = field_type
            .arguments()
            .into_iter()
            .find_map(|s| s.into_iter().last())
        {
            bound = quote! { VALUE: ::std::convert::Into<#arg> };

            argument = quote! { value: ::std::option::Option<VALUE> };

            assignment = quote! {
                self.#field_name = value.map(|v| v.into());
            };
        }
    }

    generics.push(bound);
    arguments.push(argument);

    attributes.push(syn::Attribute::from_token_stream(quote!(#[inline(always)])).unwrap());

    quote! {
        #(#attributes)*
        #[doc = #method_docs]
        pub fn #method_name <#(#generics),*> ( #(#arguments),* ) -> #return_type {
            #assignment
            self
        }
    }
}

pub(crate) fn common_get_set(field: &Field) -> TokenStream {
    let field_type = &field.ty;
    let Some(field_name) = field.ident.as_ref() else { unreachable!("unnamed field guard failed"); };

    let get = get(field_name, &field_type);
    let set = set(field_name, &field_type);
    let get_mut = get_mut(field_name, &field_type);

    quote! {
        #get
        #get_mut
        #set
    }
}

#[allow(dead_code)]
pub(crate) fn default_impl(
    struct_ident: &Ident,
    pk_field_ident: &Ident,
    fields: &Punctuated<Field, Comma>,
) -> TokenStream {
    let idents = fields.iter().filter_map(|field| {
        let ident = field.ident.as_ref().unwrap();
        (ident != pk_field_ident).then_some(ident)
    });

    quote! {
        impl Default for #struct_ident {
            fn default() -> Self {
                Self {
                    #pk_field_ident: ::rusty_ulid::generate_ulid_string(),
                    #(#idents: Default::default(),)*
                }
            }
        }
    }
}
