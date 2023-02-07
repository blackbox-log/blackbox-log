mod r#enum;
mod flag_set;

use std::collections::HashMap;

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;

use self::r#enum::Enum;
use self::flag_set::{FlagSet, Flags};

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
}

#[derive(Debug)]
struct CombinedVariant {
    official: String,
    rust: String,
    betaflight: Option<u32>,
    inav: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Firmware {
    Both,
    Betaflight,
    Inav,
}

fn combine_flags(
    betaflight: &HashMap<String, u32>,
    inav: &HashMap<String, u32>,
    rename: &HashMap<String, String>,
) -> (Vec<CombinedVariant>, Vec<Ident>, Vec<String>) {
    let get_rust_name = |official: &str| {
        rename
            .get(official)
            .cloned()
            .unwrap_or_else(|| official.to_upper_camel_case())
    };

    let mut combined = HashMap::new();

    for (official, &index) in betaflight {
        let rust = get_rust_name(official);
        assert!(combined.get(&rust).is_none());
        combined.insert(
            rust.clone(),
            CombinedVariant {
                official: official.clone(),
                rust,
                betaflight: Some(index),
                inav: None,
            },
        );
    }

    for (official, &index) in inav {
        let rust = get_rust_name(official);
        if let Some(combined) = combined.get_mut(&rust) {
            assert_eq!(&combined.official, official);
            assert!(combined.inav.is_none());
            combined.inav = Some(index);
        } else {
            combined.insert(
                rust.clone(),
                CombinedVariant {
                    official: official.clone(),
                    rust,
                    betaflight: None,
                    inav: Some(index),
                },
            );
        }
    }

    let mut combined = combined.into_values().collect::<Vec<_>>();
    combined.sort_unstable_by_key(|flag| flag.rust.clone());

    let (idents, official) = combined
        .iter()
        .map(|variant| (format_ident!("{}", variant.rust), variant.official.clone()))
        .unzip();

    (combined, idents, official)
}

#[allow(single_use_lifetimes)]
fn expand_combined_flags<'f, 'i>(
    name: &Ident,
    flags: impl IntoIterator<Item = &'f CombinedVariant>,
    idents: impl IntoIterator<Item = &'i Ident>,
    unknown: bool,
) -> TokenStream {
    let body = flags
        .into_iter()
        .zip(idents.into_iter())
        .map(|(flag, ident)| {
            let note = match (flag.betaflight.is_some(), flag.inav.is_some()) {
                (true, true) => "",
                (true, false) => " (Betaflight only)",
                (false, true) => " (INAV only)",
                _ => unreachable!(),
            };

            let doc = format!("`{}`{note}", flag.official);
            quote! { #[doc = #doc] #ident }
        });

    let unknown = unknown.then_some(quote!(Unknown));

    quote! {
        pub enum #name {
            #(#body, )*
            #unknown
        }
    }
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

#[allow(single_use_lifetimes)]
fn impl_flag<'v>(
    name: &Ident,
    variants: impl IntoIterator<Item = &'v Ident>,
    names: &[String],
    unknown: bool,
) -> TokenStream {
    let variants = variants.into_iter();
    let unknown = unknown.then_some(quote!(Self::Unknown => "UNKNOWN"));
    quote! {
        #[allow(unused_qualifications)]
        impl crate::units::Flag for #name {
            fn as_name(&self) -> &'static str {
                match self {
                    #( Self::#variants => #names, )*
                    #unknown
                }
            }
        }
    }
}

fn impl_flag_display(name: &Ident) -> TokenStream {
    quote! {
        impl ::core::fmt::Display for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let s = <Self as crate::units::Flag>::as_name(self);
                f.write_str(s)
            }
        }
    }
}
