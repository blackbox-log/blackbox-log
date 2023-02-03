use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

use super::{
    combine_flags, expand_combined_flags, impl_flag, impl_flag_display, quote_attrs, Firmware,
};

#[derive(Debug, Deserialize)]
pub struct FlagSet {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
}

impl FlagSet {
    pub fn expand(&self, flag_name: &str) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let attrs = quote_attrs(&self.doc, &self.attrs);
        let flag_name = format_ident!("{}", flag_name);

        quote! {
            #attrs
            #[allow(unused_qualifications)]
            pub struct #name {
                firmware: crate::headers::Firmware,
                raw: ::bitvec::array::BitArray<[u32; 1], ::bitvec::order::Lsb0>
            }

            #[allow(unused_qualifications)]
            impl #name {
                pub(crate) fn new(raw: u32, firmware: crate::headers::Firmware) -> Self {
                    Self {
                        firmware,
                        raw: ::bitvec::array::BitArray::new([raw])
                    }
                }
            }

            #[allow(unused_qualifications, clippy::cast_possible_truncation)]
            impl crate::units::FlagSet for #name {
                type Flag = #flag_name;

                fn is_set(&self, flag: Self::Flag) -> bool {
                    flag.to_bit(self.firmware).map_or(false, |bit| self.raw[bit as usize])
                }

                fn as_names(&self) -> ::alloc::vec::Vec<&'static str> {
                    self.raw
                        .iter_ones()
                        .filter_map(|bit| {
                            let flag = <#flag_name>::from_bit(bit as u32, self.firmware)?;
                            let name = <#flag_name as crate::units::Flag>::as_name(&flag);
                            Some(name)
                        })
                        .collect()
                }
            }

            impl ::core::fmt::Display for #name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    let names = <Self as crate::units::FlagSet>::as_names(self);
                    f.write_str(&names.join("|"))
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
    betaflight: HashMap<String, u32>,
    inav: HashMap<String, u32>,
    #[serde(default)]
    rename: HashMap<String, String>,
}

impl Flags {
    pub fn expand(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let attrs = quote_attrs(&self.doc, &self.attrs);

        let (flags, idents, official) = combine_flags(&self.betaflight, &self.inav, &self.rename);
        let enum_def = expand_combined_flags(&name, &flags, &idents, false);
        let impl_flag = impl_flag(&name, &idents, &official, false);
        let impl_flag_display = impl_flag_display(&name);

        let mut from_bit = Vec::new();
        for (flag, ident) in flags.iter().zip(idents.iter()) {
            if flag.betaflight == flag.inav && flag.betaflight.is_some() {
                let bit = flag.betaflight.unwrap();
                let arm = quote!((#bit, _) => Some(Self::#ident));
                from_bit.push((bit, Firmware::Both, arm));
                continue;
            }

            if let Some(bit) = flag.betaflight {
                let arm = quote!((#bit, Betaflight(_)) => Some(Self::#ident));
                from_bit.push((bit, Firmware::Betaflight, arm));
            }

            if let Some(bit) = flag.inav {
                let arm = quote!((#bit, Inav(_)) => Some(Self::#ident));
                from_bit.push((bit, Firmware::Inav, arm));
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
                to_bit.push(quote!((Self::#ident, Betaflight(_)) => Some(#bit)));
            }

            if let Some(bit) = flag.inav {
                to_bit.push(quote!((Self::#ident, Inav(_)) => Some(#bit)));
            }
        }

        quote! {
            #attrs
            #enum_def
            #impl_flag
            #impl_flag_display

            #[allow(unused_imports, unused_qualifications, clippy::match_same_arms, clippy::unseparated_literal_suffix)]
            impl #name {
                const fn from_bit(bit: u32, firmware: crate::headers::Firmware) -> Option<Self> {
                    use crate::headers::Firmware::{Betaflight, Inav};
                    match (bit, firmware) {
                        #(#from_bit,)*
                        _ => None
                    }
                }

                const fn to_bit(self, firmware: crate::headers::Firmware) -> Option<u32> {
                    use crate::headers::Firmware::{Betaflight, Inav};
                    match (self, firmware) {
                        #(#to_bit,)*
                        _ => None
                    }
                }
            }

        }
    }
}
