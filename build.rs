use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use glob::glob;
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use serde::de::Visitor;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TypeDef {
    Flags { set: FlagSet, flags: Flags },
    Enum { r#enum: Enum },
}

impl TypeDef {
    fn expand(&self) -> TokenStream {
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

#[derive(Debug, Deserialize)]
struct FlagSet {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
}

impl FlagSet {
    fn expand(&self, flag_name: &str) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let attrs = quote_attrs(&self.doc, &self.attrs);
        let flag_name = format_ident!("{}", flag_name);

        quote! {
            #attrs
            #[allow(unused_qualifications)]
            pub struct #name {
                firmware: crate::headers::FirmwareKind,
                raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>
            }

            #[allow(unused_qualifications)]
            impl #name {
                pub(crate) fn new(raw: u32, firmware: crate::headers::FirmwareKind) -> Self {
                    Self {
                        firmware,
                        raw: ::bitvec::array::BitArray::new([raw])
                    }
                }
            }

            #[allow(unused_qualifications)]
            impl crate::units::FlagSet for #name {
                type Flag = #flag_name;

                fn is_set(&self, flag: Self::Flag) -> bool {
                    flag.to_bit(self.firmware).map_or(false, |bit| self.raw[bit as usize])
                }

                fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
                    self.raw
                        .iter_ones()
                        .filter_map(|bit| Some(<#flag_name>::from_bit(bit as u32, self.firmware)?.as_name()))
                        .collect()
                }
            }

            impl ::core::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.write_str(&self.as_names().join("|"))
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct Flags {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
    betaflight: Vec<Variant>,
    inav: Vec<Variant>,
}

impl Flags {
    fn expand(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let attrs = quote_attrs(&self.doc, &self.attrs);

        let (flags, idents, official) = combine_flags(&self.betaflight, &self.inav);
        let enum_def = expand_combined_flags(&name, &flags, &idents, false);
        let impl_flag = impl_flag(&name, &idents, &official, false);
        let impl_flag_display = impl_flag_display(&name);

        let mut from_bit = Vec::new();
        for (flag, ident) in flags.iter().zip(idents.iter()) {
            if flag.betaflight == flag.inav && flag.betaflight.is_some() {
                let bit = flag.betaflight.unwrap();
                let arm = quote!((#bit, _) => Some(Self::#ident));
                from_bit.push((bit, FirmwareKind::Both, arm));
                continue;
            }

            if let Some(bit) = flag.betaflight {
                let arm = quote!((#bit, Betaflight | EmuFlight) => Some(Self::#ident));
                from_bit.push((bit, FirmwareKind::Betaflight, arm));
            }

            if let Some(bit) = flag.inav {
                let arm = quote!((#bit, Inav) => Some(Self::#ident));
                from_bit.push((bit, FirmwareKind::Inav, arm));
            }
        }
        from_bit.sort_unstable_by_key(|(index, firmware, _)| (*index, *firmware));
        let from_bit = from_bit.iter().map(|(_, _, arm)| arm);

        let mut to_bit = Vec::new();
        for (flag, ident) in flags.iter().zip(idents.iter()) {
            if flag.betaflight == flag.inav && flag.betaflight.is_some() {
                let bit = flag.betaflight.unwrap();
                to_bit.push(quote!((Self::#ident, _) => Some(#bit)));
                continue;
            }

            if let Some(bit) = flag.betaflight {
                to_bit.push(quote!((Self::#ident, Betaflight | EmuFlight) => Some(#bit)));
            }

            if let Some(bit) = flag.inav {
                to_bit.push(quote!((Self::#ident, Inav) => Some(#bit)));
            }
        }

        quote! {
            #attrs
            #enum_def
            #impl_flag
            #impl_flag_display

            #[allow(clippy::match_same_arms, unused_qualifications)]
            impl #name {
                const fn from_bit(bit: u32, firmware: crate::headers::FirmwareKind) -> Option<Self> {
                    use crate::headers::FirmwareKind::{Betaflight, EmuFlight, Inav};
                    match (bit, firmware) {
                        #(#from_bit,)*
                        _ => None
                    }
                }

                const fn to_bit(self, firmware: crate::headers::FirmwareKind) -> Option<u32> {
                    use crate::headers::FirmwareKind::{Betaflight, EmuFlight, Inav};
                    match (self, firmware) {
                        #(#to_bit,)*
                        _ => None
                    }
                }
            }

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
enum FirmwareKind {
    Both,
    Betaflight,
    Inav,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Enum {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
    unknown: bool,
    unknown_callback: Option<String>,
    betaflight: Vec<Variant>,
    inav: Vec<Variant>,
}

impl Enum {
    fn expand(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let attrs = quote_attrs(&self.doc, &self.attrs);

        let (variants, idents, official) = combine_flags(&self.betaflight, &self.inav);

        let enum_def = expand_combined_flags(&name, &variants, &idents, self.unknown);
        let impl_flag = impl_flag(&name, &idents, &official, self.unknown);
        let impl_flag_display = impl_flag_display(&name);

        let (return_type, default) = if self.unknown {
            (quote!(Self), quote!(Self::Unknown))
        } else {
            (quote!(Option<Self>), quote!(None))
        };

        let unknown_cb = self.unknown_callback.as_deref().unwrap_or("|_| ()");
        let unknown_cb = syn::parse_str::<syn::ExprClosure>(unknown_cb).unwrap();

        let mut new = Vec::new();
        for (variant, ident) in variants.iter().zip(idents.iter()) {
            let value = quote!(Self::#ident);
            let value = if self.unknown {
                value
            } else {
                quote!(Some(#value))
            };

            if variant.betaflight == variant.inav && variant.betaflight.is_some() {
                let index = variant.betaflight.unwrap();
                let arm = quote!((#index, _) => #value);
                new.push((index, FirmwareKind::Both, arm));
                continue;
            }

            if let Some(index) = variant.betaflight {
                let arm = quote!((#index, Betaflight | EmuFlight) => #value);
                new.push((index, FirmwareKind::Betaflight, arm));
            }

            if let Some(index) = variant.inav {
                let arm = quote!((#index, Inav) => #value);
                new.push((index, FirmwareKind::Inav, arm));
            }
        }
        new.sort_unstable_by_key(|(index, firmware, _)| (*index, *firmware));
        let new = new.iter().map(|(_, _, arm)| arm);

        quote! {
            #attrs
            #enum_def
            #impl_flag
            #impl_flag_display

            #[allow(clippy::match_same_arms, unused_qualifications)]
            impl #name {
                pub(crate) fn new(raw: u32, firmware: crate::headers::FirmwareKind) -> #return_type {
                    use crate::headers::FirmwareKind::{Betaflight, EmuFlight, Inav};
                    match (raw, firmware) {
                        #(#new,)*
                        _ => {
                            (#unknown_cb)(raw);
                            #default
                        }
                    }
                }
            }
        }
    }
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

fn main() {
    println!("cargo:rerun-if-changed=types/");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = std::path::PathBuf::from(out_dir);

    let mut files = Vec::new();

    for f in glob("types/*.yaml").unwrap() {
        let f = f.unwrap();
        let filename = f.file_stem().unwrap();

        let mut out_path = out_dir.clone();
        out_path.push(filename);
        out_path.set_extension("rs");
        let mut out = File::create(&out_path).unwrap();
        files.push(out_path);

        let f = File::open(f).unwrap();
        let s = std::io::read_to_string(f).unwrap();
        let type_def: TypeDef = serde_yaml::from_str(&s).unwrap();

        let tokens = type_def.expand();
        writeln!(out, "{tokens}").unwrap();
    }

    if Command::new("rustfmt")
        .arg("+nightly")
        .args(files)
        .status()
        .map(|status| status.success())
        .ok()
        != Some(true)
    {
        println!("cargo:warning=failed to run `rustfmt +nightly` on generated files");
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

            fn expecting(&self, _formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                todo!()
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
