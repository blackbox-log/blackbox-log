use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;

use super::{impl_flag, quote_attrs, str_to_ident, AugmentedData};
use crate::type_def::{flag_docs, get_as_name_arms};

#[derive(Debug, Deserialize)]
pub struct FlagSet {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
}

impl FlagSet {
    pub fn expand(&self, flag_name: &str) -> TokenStream {
        let doc = format!(
            "{}\n\nSee [`FlagSet`][crate::units::FlagSet] and [`{flag_name}`].",
            self.doc
        );
        let attrs = quote_attrs(&doc, &self.attrs, false);

        let name = format_ident!("{}", self.name);
        let flag_name = format_ident!("{}", flag_name);

        quote! {
            #attrs
            #[allow(unused_qualifications)]
            pub struct #name {
                firmware: crate::headers::InternalFirmware,
                raw: ::bitvec::array::BitArray<u32, ::bitvec::order::Lsb0>
            }

            #[allow(unused_qualifications, clippy::cast_possible_truncation)]
            impl #name {
                pub(crate) fn new(raw: u32, firmware: crate::headers::InternalFirmware) -> Self {
                    Self {
                        firmware,
                        raw: ::bitvec::array::BitArray::new(raw)
                    }
                }

                fn iter(&self) -> impl Iterator<Item = <Self as crate::units::FlagSet>::Flag> + '_ {
                    self.raw
                        .iter_ones()
                        .filter_map(|bit| <#flag_name>::from_bit(bit as u32, self.firmware))
                }
            }

            #[allow(unused_qualifications, clippy::cast_possible_truncation)]
            impl crate::units::FlagSet for #name {
                type Flag = #flag_name;

                fn is_set(&self, flag: Self::Flag) -> bool {
                    flag.to_bit(self.firmware).map_or(false, |bit| self.raw[bit as usize])
                }

                fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
                    self.iter()
                        .map(|flag| <#flag_name as crate::units::Flag>::as_name(&flag))
                        .collect()
                }
            }

            #[allow(unused_qualifications)]
            impl ::core::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let names = <Self as crate::units::FlagSet>::as_names(self);
                    f.write_str(&names.join("|"))
                }
            }

            #[cfg(feature = "_serde")]
            #[allow(clippy::cast_possible_truncation)]
            impl ::serde::Serialize for #name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    use serde::ser::SerializeSeq;

                    // TODO: length
                    let mut seq = serializer.serialize_seq(None)?;

                    for flag in self.iter() {
                        seq.serialize_element(&flag)?;
                    }

                    seq.end()
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Flags {
    pub(super) name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
    #[serde(default)]
    rename: HashMap<String, String>,
    #[serde(default)]
    pub(super) data: HashMap<String, HashMap<u32, Vec<String>>>,
}

impl Flags {
    pub fn expand(&self) -> TokenStream {
        let name = self.name();
        let attrs = quote_attrs(&flag_docs(&self.doc), &self.attrs, true);

        let (data, flags) = super::augment_data(&self.data, &self.rename);
        let bit_conversions = self.bit_conversions(&data);

        let as_name = get_as_name_arms(&flags);
        let impl_flag = impl_flag(&name, as_name);

        let enum_def = flags.iter().map(|(official, rust)| {
            let doc = format!("`{official}`");
            quote!(#[doc = #doc] #rust)
        });

        quote! {
            #attrs
            #[non_exhaustive]
            pub enum #name {
                #(#enum_def),*
            }

            #impl_flag
            #bit_conversions
        }
    }

    fn name(&self) -> Ident {
        str_to_ident(&self.name)
    }

    fn bit_conversions(&self, data: &AugmentedData<'_>) -> TokenStream {
        let name = self.name();

        let mut from_bit = Vec::new();
        for ((_, rust), indices) in data {
            for (index, firmwares) in indices.iter() {
                let key = (index, firmwares[0].clone());
                from_bit.push((
                    key,
                    quote! {
                        (#index, #(#firmwares)|*) => Some(Self::#rust)
                    },
                ));
            }
        }
        from_bit.sort_unstable_by_key(|(key, _)| key.clone());
        let from_bit = from_bit.into_iter().map(|(_, tokens)| tokens);

        let mut to_bit = Vec::new();
        for ((_, rust), indices) in data {
            for (index, firmwares) in indices.iter() {
                let key = (index, firmwares[0].clone());
                to_bit.push((
                    key,
                    quote! {
                        (Self::#rust, #(#firmwares)|*) => Some(#index)
                    },
                ));
            }
        }
        to_bit.sort_unstable_by_key(|(key, _)| key.clone());
        let to_bit = to_bit.into_iter().map(|(_, tokens)| tokens);

        quote! {
            #[allow(unused_qualifications, clippy::enum_glob_use, clippy::match_same_arms, clippy::unseparated_literal_suffix)]
            impl #name {
                const fn from_bit(bit: u32, fw: crate::headers::InternalFirmware) -> Option<Self> {
                    use crate::headers::InternalFirmware::*;
                    match (bit, fw) {
                        #(#from_bit,)*
                        _ => None
                    }
                }

                const fn to_bit(self, fw: crate::headers::InternalFirmware) -> Option<u32> {
                    use crate::headers::InternalFirmware::*;
                    match (self, fw) {
                        #(#to_bit,)*
                        _ => None
                    }
                }
            }
        }
    }
}
