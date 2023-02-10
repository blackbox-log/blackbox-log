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
    rust: Ident,
    betaflight: Option<u32>,
    inav: Option<u32>,
}

impl CombinedVariant {
    pub const fn firmware(&self) -> Firmware {
        match (self.betaflight.is_some(), self.inav.is_some()) {
            (true, true) => Firmware::Both,
            (true, false) => Firmware::Betaflight,
            (false, true) => Firmware::Inav,
            (false, false) => unreachable!(),
        }
    }
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
) -> Vec<CombinedVariant> {
    fn str_to_ident(s: &str) -> Ident {
        format_ident!("{}", s)
    }

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
                rust: str_to_ident(&rust),
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
                    rust: str_to_ident(&rust),
                    betaflight: None,
                    inav: Some(index),
                },
            );
        }
    }

    let mut combined = combined.into_values().collect::<Vec<_>>();
    combined.sort_unstable_by_key(|flag| flag.rust.clone());
    combined
}

#[allow(single_use_lifetimes)]
fn expand_combined_flags<'f, 'i>(
    name: &Ident,
    flags: impl IntoIterator<Item = &'f CombinedVariant>,
    unknown: bool,
) -> TokenStream {
    let body = flags.into_iter().map(|flag| {
        let note = match flag.firmware() {
            Firmware::Both => "",
            Firmware::Betaflight => " (Betaflight only)",
            Firmware::Inav => " (INAV only)",
        };

        let doc = format!("`{}`{note}", flag.official);
        let ident = &flag.rust;
        quote!(#[doc = #doc] #ident)
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
    flags: impl IntoIterator<Item = &'v CombinedVariant>,
    unknown: bool,
) -> TokenStream {
    let variants = flags.into_iter().map(|flag| {
        let variant = &flag.rust;
        let official = &flag.official;

        quote!(Self::#variant => #official)
    });

    let unknown = unknown.then_some(quote!(Self::Unknown => "UNKNOWN"));
    quote! {
        #[allow(unused_qualifications)]
        impl crate::units::Flag for #name {
            fn as_name(&self) -> &'static str {
                match self {
                    #(#variants,)*
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
