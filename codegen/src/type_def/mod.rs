mod r#enum;
mod flag_set;

use std::collections::HashMap;

use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use serde::de::Visitor;
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
struct Variant {
    official: String,
    rust: String,
    index: u32,
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
    betaflight: &[Variant],
    inav: &[Variant],
) -> (Vec<CombinedVariant>, Vec<Ident>, Vec<String>) {
    let mut combined = HashMap::new();

    for flag in betaflight {
        assert!(combined.get(&flag.rust).is_none());
        combined.insert(
            &flag.rust,
            CombinedVariant {
                official: flag.official.clone(),
                rust: flag.rust.clone(),
                betaflight: Some(flag.index),
                inav: None,
            },
        );
    }

    for flag in inav {
        if let Some(combined) = combined.get_mut(&flag.rust) {
            assert_eq!(combined.official, flag.official);
            assert!(combined.inav.is_none());
            combined.inav = Some(flag.index);
        } else {
            combined.insert(
                &flag.rust,
                CombinedVariant {
                    official: flag.official.clone(),
                    rust: flag.rust.clone(),
                    betaflight: None,
                    inav: Some(flag.index),
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

fn quote_attrs(doc: &str, attrs: &[String]) -> TokenStream {
    let doc = doc.lines();
    let attrs = attrs
        .iter()
        .map(|attr| syn::parse_str::<syn::Meta>(attr).unwrap());

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
                f.write_str(self.as_name())
            }
        }
    }
}

impl<'de> Deserialize<'de> for Variant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess};

        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Official,
            Rust,
            Index,
        }

        struct FlagVisitor;

        impl<'de> Visitor<'de> for FlagVisitor {
            type Value = Variant;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "a map with keys `official`, `rust`, and `index`, or a map with one entry \
                     `official` -> `index`"
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                if map.size_hint() == Some(1) {
                    let (official, index): (String, u32) = map.next_entry()?.unwrap();
                    let rust = official.to_upper_camel_case();

                    Ok(Variant {
                        official,
                        rust,
                        index,
                    })
                } else {
                    let mut official = None;
                    let mut rust = None;
                    let mut index = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Official => {
                                if official.is_some() {
                                    return Err(Error::duplicate_field("official"));
                                }
                                official = Some(map.next_value()?);
                            }
                            Field::Rust => {
                                if rust.is_some() {
                                    return Err(Error::duplicate_field("rust"));
                                }
                                rust = Some(map.next_value()?);
                            }
                            Field::Index => {
                                if index.is_some() {
                                    return Err(Error::duplicate_field("index"));
                                }
                                index = Some(map.next_value()?);
                            }
                        }
                    }

                    let official = official.ok_or_else(|| Error::missing_field("official"))?;
                    let rust = rust.ok_or_else(|| Error::missing_field("rust"))?;
                    let index = index.ok_or_else(|| Error::missing_field("index"))?;

                    Ok(Variant {
                        official,
                        rust,
                        index,
                    })
                }
            }
        }

        deserializer.deserialize_map(FlagVisitor)
    }
}
