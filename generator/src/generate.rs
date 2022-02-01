use std::error::Error;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::{category::Category, renames};

pub fn category_to_tokens(
    category_enum_name: &str,
    category: &Category,
) -> Result<TokenStream, Box<dyn Error>> {
    // Apply renames
    let enum_name = renames::RENAMES
        .iter()
        .find(|rename| rename.name == category_enum_name)
        .and_then(|rename| rename.rename_to)
        .unwrap_or(category_enum_name);

    let enum_name = Ident::new(enum_name, Span::call_site());

    let constant_tokens = category
        .constants
        .iter()
        .map(|constant| {
            let alias = &constant.alias_name;
            let constant_name = Ident::new(&constant.name, Span::call_site());
            let comment = constant
                .comment
                .as_ref()
                .map(|comment| quote! { #[doc = #comment] });
            let value = constant.value;

            quote! {
                #comment
                #[doc(alias = #alias)]
                pub const #constant_name: #enum_name = #enum_name(#value);
            }
        })
        .collect::<Vec<_>>();

    let tokens = quote! {
        #[repr(transparent)]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            // The implementation of Ord may not mean anything
            PartialOrd,
            Ord,
            Hash
        )]
        pub struct #enum_name (u32);

        impl #enum_name {
            pub const fn new(value: u32) -> Self {
                Self(value)
            }

            pub const fn into_inner(self) -> u32 {
                self.0
            }
        }

        impl From<u32> for #enum_name {
            fn from(value: u32) -> Self {
                Self(value)
            }
        }

        impl From<#enum_name> for u32 {
            fn from(value: #enum_name) -> u32 {
                value.0
            }
        }

        impl AsRef<u32> for #enum_name {
            fn as_ref(&self) -> &u32 {
                &self.0
            }
        }

        impl ::core::borrow::Borrow<u32> for #enum_name {
            fn borrow(&self) -> &u32 {
                &self.0
            }
        }

        impl PartialEq<u32> for #enum_name {
            fn eq(&self, other: &u32) -> bool {
                &self.0 == other
            }
        }

        impl #enum_name {
            #(#constant_tokens)*
        }
    };

    Ok(tokens)
}
