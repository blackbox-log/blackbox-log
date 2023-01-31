use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::{env, io};

use glob::glob;
use heck::ToUpperCamelCase;
use quote::{format_ident, quote};
use serde::de::Visitor;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TypeDef {
    Flags { set: FlagSet, flags: Flags },
}

impl TypeDef {
    fn write(&self, out: &mut impl Write) -> io::Result<()> {
        match self {
            Self::Flags { set, flags } => {
                set.write(out, &flags.name)?;
                flags.write(out)
            }
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
    fn write(&self, out: &mut impl Write, flag_name: &str) -> io::Result<()> {
        let Self { name, doc, attrs } = self;
        let name = format_ident!("{}", name);
        let doc = doc.lines();
        let attrs = attrs
            .iter()
            .map(|attr| syn::parse_str::<syn::Meta>(attr).unwrap());
        let flag_name = format_ident!("{}", flag_name);

        let tokens = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #( #[doc = #doc] )*
            #( #[#attrs] )*
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
        };

        writeln!(out, "{tokens}")
    }
}

#[derive(Debug, Deserialize)]
struct Flags {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
    betaflight: Vec<Flag>,
    inav: Vec<Flag>,
}

impl Flags {
    fn write(&self, out: &mut impl Write) -> io::Result<()> {
        let Self {
            name, doc, attrs, ..
        } = self;
        let name = format_ident!("{}", name);
        let doc = doc.lines();
        let attrs = attrs
            .iter()
            .map(|attr| syn::parse_str::<syn::Meta>(attr).unwrap());

        let flags = self.combined_flags();
        let ident_flags = flags
            .iter()
            .map(|flag| (format_ident!("{}", flag.rust), flag))
            .collect::<Vec<_>>();

        let flag_names = ident_flags
            .iter()
            .map(|(ident, _)| ident)
            .collect::<Vec<_>>();
        let official_names = flags.iter().map(|flag| &flag.official).collect::<Vec<_>>();

        let enum_body = ident_flags.iter().map(|(ident, flag)| {
            let note = match (flag.betaflight.is_some(), flag.inav.is_some()) {
                (true, true) => "",
                (true, false) => " (Betaflight only)",
                (false, true) => " (INAV only)",
                _ => unreachable!(),
            };

            let doc = format!("`{}`{note}", flag.official);
            quote! { #[doc = #doc] #ident }
        });

        let mut from_bit = Vec::new();
        for (ident, flag) in &ident_flags {
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
        for (ident, flag) in &ident_flags {
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

        let tokens = quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #( #[doc = #doc] )*
            #( #[#attrs] )*
            pub enum #name {
                #(#enum_body),*
            }

            #[allow(unused_qualifications)]
            impl crate::units::Flag for #name {
                fn as_name(&self) -> &'static str {
                    match self {
                        #( Self::#flag_names => #official_names ),*
                    }
                }
            }

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

            impl ::core::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    f.write_str(self.as_name())
                }
            }
        };

        writeln!(out, "{tokens}")
    }

    fn combined_flags(&self) -> Vec<CombinedFlag> {
        let mut combined = HashMap::new();

        for flag in &self.betaflight {
            assert!(combined.get(&flag.rust).is_none());
            combined.insert(
                &flag.rust,
                CombinedFlag {
                    official: flag.official.clone(),
                    rust: flag.rust.clone(),
                    betaflight: Some(flag.index),
                    inav: None,
                },
            );
        }

        for flag in &self.inav {
            if let Some(combined) = combined.get_mut(&flag.rust) {
                assert_eq!(combined.official, flag.official);
                assert!(combined.inav.is_none());
                combined.inav = Some(flag.index);
            } else {
                combined.insert(
                    &flag.rust,
                    CombinedFlag {
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
        combined
    }
}

#[derive(Debug)]
struct Flag {
    official: String,
    rust: String,
    index: u32,
}

#[derive(Debug)]
struct CombinedFlag {
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

        type_def.write(&mut out).unwrap();
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

impl<'de> Deserialize<'de> for Flag {
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
            type Value = Flag;

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

                    Ok(Flag {
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

                    Ok(Flag {
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
