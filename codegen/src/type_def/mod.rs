mod r#enum;
mod flag_set;

use std::collections::HashMap;

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;

use self::flag_set::{FlagSet, Flags};
use self::r#enum::Enum;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TypeDef {
    Flags { set: FlagSet, flags: Flags },
    Enum { r#enum: Enum },
}

impl TypeDef {
    pub fn expand(&self) -> TokenStream {
        match self {
            Self::Flags { set, flags } => {
                let mut tokens = set.expand(&flags.name);
                tokens.extend(flags.expand());
                tokens
            }
            Self::Enum { r#enum } => r#enum.expand(),
        }
    }

    pub fn add_data(&mut self, firmware: String, new_data: HashMap<String, u32>) {
        let (Self::Flags {
            flags: Flags { data, .. },
            ..
        }
        | Self::Enum {
            r#enum: Enum { data, .. },
        }) = self;

        for (official, index) in new_data {
            let indices = data.entry(official).or_default();
            let firmwares = indices.entry(index).or_default();
            let i = firmwares.partition_point(|s| s < &firmware);
            firmwares.insert(i, firmware.clone());
        }
    }
}

fn flag_docs(doc: &str) -> String {
    format!("{doc}\n\nSee [`Flag`][crate::units::Flag].")
}

fn quote_attrs(doc: &str, attrs: &[String], serde: bool) -> TokenStream {
    let serde = serde.then_some(quote!(#[cfg_attr(feature = "_serde", derive(serde::Serialize))]));
    let doc = doc.lines();
    let attrs = attrs
        .iter()
        .map(|attr| syn::parse_str::<syn::Meta>(attr).unwrap());

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #serde
        #( #[doc = #doc] )*
        #( #[#attrs] )*
    }
}

type AugmentedData<'a> = HashMap<(&'a str, Ident), HashMap<u32, Vec<Ident>>>;

fn augment_data<'d>(
    data: &'d HashMap<String, HashMap<u32, Vec<String>>>,
    rename: &HashMap<String, String>,
) -> (AugmentedData<'d>, Vec<(&'d str, Ident)>) {
    let data = data
        .iter()
        .map(|(official, indices)| {
            let rust = rename
                .get(official)
                .cloned()
                .unwrap_or_else(|| official.to_upper_camel_case());
            let rust = str_to_ident(&rust);

            let indices = indices
                .iter()
                .map(|(i, fw)| (*i, fw.iter().map(|s| str_to_ident(s)).collect::<Vec<_>>()))
                .collect();

            ((official.as_str(), rust), indices)
        })
        .collect::<HashMap<_, _>>();

    let mut variants = data
        .keys()
        .map(|(official, rust)| (*official, rust.clone()))
        .collect::<Vec<_>>();
    variants.sort_unstable_by_key(|(_, rust)| rust.clone());

    (data, variants)
}

fn get_as_name_arms<'a>(
    variants: &'a [(&'a str, Ident)],
) -> impl Iterator<Item = TokenStream> + 'a {
    variants
        .iter()
        .map(|(official, rust)| quote!(Self::#rust => #official))
}

fn impl_flag(name: &Ident, as_name: impl Iterator<Item = TokenStream>) -> TokenStream {
    quote! {
        #[allow(unused_qualifications)]
        impl crate::units::Flag for #name {
            fn as_name(&self) -> &'static str {
                match self {
                    #(#as_name),*
                }
            }
        }

        #[allow(unused_qualifications)]
        impl ::core::fmt::Display for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let s = <Self as crate::units::Flag>::as_name(self);
                f.write_str(s)
            }
        }
    }
}

fn str_to_ident(s: &str) -> Ident {
    format_ident!("{}", s)
}
