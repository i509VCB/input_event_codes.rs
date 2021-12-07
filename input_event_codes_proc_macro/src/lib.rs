extern crate proc_macro_error;

use std::collections::HashSet;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use proc_macro_error::{abort_call_site, proc_macro_error, ResultExt};
use quote::quote;
use syn::parse::{ParseBuffer, Parser};

/// Generate enums representing all the input event codes.
///
/// This macro will create an enum for every entry inside `input-event-codes.h`.
#[proc_macro]
#[proc_macro_error]
pub fn generate_input_codes(tokens: TokenStream) -> TokenStream {
    // TODO: Change this, we want to be able to attach docs if possible.
    if !tokens.is_empty() {
        proc_macro_error::abort_call_site!("generate_input_codes! does not take any parameters");
    }

    let generated = match bindgen::Builder::default()
        .header("./input-event-codes.h")
        // The header will only contain defines.
        // The header does say there may be typedefs but none are declared.
        .ignore_functions()
        .ignore_methods()
        .rustfmt_bindings(true)
        .generate()
    {
        Ok(generated) => generated.to_string(),

        // Bindgen should provide a nicer error in the future: https://github.com/rust-lang/rust-bindgen/pull/2125
        Err(_) => proc_macro_error::abort_call_site!(
            "Failed to generate bindings for input-event-codes.h"
        ),
    };

    // Now perform the incredibly cursed parsing the source from bindgen to create a more idomatic api.
    // TODO: Could bindgen give us the token stream directly in the future?
    let tokens: TokenStream = match generated.parse() {
        Ok(stream) => stream,
        // proc_macro_error does not implement `Into<Diagnostic> for LexError`
        Err(err) => {
            proc_macro_error::abort_call_site!("{}", err);
        }
    };

    // Parse every single const item in the file, there should only be const items.
    let mut consts = Parser::parse(
        |buffer: &ParseBuffer<'_>| -> syn::Result<Vec<syn::ItemConst>> {
            let mut items = vec![];

            while !buffer.is_empty() {
                items.push(buffer.parse()?);
            }

            Ok(items)
        },
        tokens,
    )
    .expect_or_abort("error parsing generated bindings");

    // Create categories of the definitions in the input event codes header.
    let categories = consts
        .iter()
        // The category name is simply everything before the first `_`,
        .map(|item| item.ident.to_string().split_once('_').unwrap().0.to_owned())
        // Collect into a hash set so we do not have 100 category names called `KEY` and only 1.
        .collect::<HashSet<_>>()
        .into_iter()
        // Now take the entries from consts and put them in their categories
        .map(|name| {
            let mut entries = vec![];

            consts.retain(|item| {
                if item.ident.to_string().starts_with(&name) {
                    entries.push(item.clone());
                    // Remove element
                    false
                } else {
                    true
                }
            });

            Category { name, entries }
        })
        .collect::<Vec<_>>();

    if !consts.is_empty() {
        panic!("Entries not fully exhausted. This is a bug!");
    }

    // Generate enums
    let mut enums = Vec::with_capacity(categories.len());

    for category in categories {
        enums.push(generate_enum(category));
    }

    TokenStream::from(quote! { #(#enums)* })
}

fn generate_enum(mut category: Category) -> proc_macro2::TokenStream {
    // TODO: Category name replacement, `Ev` is a badly named enum for example.

    // Normalize the name to Rust standards.
    category.name.get_mut(0..1).unwrap().make_ascii_uppercase();
    category.name.get_mut(1..).unwrap().make_ascii_lowercase();

    let name = Ident::new(&category.name, Span::call_site());

    let entries = category
        .entries
        .into_iter()
        .map(|item| {
            let mut name = item
                .ident
                .to_string()
                .split('_')
                .skip(1) // Skip the category name
                // Combine all segments
                .collect::<String>();

            // Prefix any entries that start with a number
            if name.starts_with(char::is_numeric) {
                name.insert(0, '_');
            }

            let ident = Ident::new(&name, Span::call_site());
            let original_name = Literal::string(&item.ident.to_string());

            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(value),
                ..
            }) = *item.expr
            {
                let value = Literal::u32_unsuffixed(value.base10_parse::<u32>().unwrap_or_abort());

                Entry {
                    ident,
                    original_name,
                    value,
                }
            } else {
                abort_call_site!("Expression value of {} is not a literal", name);
            }
        })
        .collect::<Vec<_>>();

    let entries = entries
        .iter()
        .map(|entry| {
            let ident = &entry.ident;
            let value = &entry.value;
            let alias = &entry.original_name;

            // With the definition, we also include the original name before any processing for easy searching.
            quote! {
                #[doc(alias = #alias)]
                pub const #ident: #name = #name(#value);
            }
        })
        .collect::<Vec<_>>();

    // TODO: Note how the ordering may not mean anything
    quote! {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #name (u32);

        impl #name {
            pub const fn new(value: u32) -> #name {
                #name(value)
            }

            pub const fn into_inner(self) -> u32 {
                self.0
            }
        }

        impl From<u32> for #name {
            fn from(value: u32) -> #name {
                #name(value)
            }
        }

        impl From<#name> for u32 {
            fn from(value: #name) -> u32 {
                value.0
            }
        }

        impl PartialEq<u32> for #name {
            fn eq(&self, other: &u32) -> bool {
                &self.0 == other
            }
        }

        impl #name {
            #(#entries)*
        }
    }
}

struct Category {
    name: String,
    entries: Vec<syn::ItemConst>,
}

struct Entry {
    ident: Ident,
    original_name: Literal,
    value: Literal,
}
