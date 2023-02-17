use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

use super::{
    combine_flags, expand_combined_flags, impl_flag, impl_flag_display, quote_attrs, Firmware,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
    unknown: bool,
    unknown_callback: Option<String>,
    betaflight: HashMap<String, u32>,
    inav: HashMap<String, u32>,
    #[serde(default)]
    rename: HashMap<String, String>,
}

impl Enum {
    pub fn expand(&self) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let attrs = quote_attrs(&self.doc, &self.attrs, true);

        let variants = combine_flags(&self.betaflight, &self.inav, &self.rename);
        let enum_def = expand_combined_flags(&name, &variants, self.unknown);
        let impl_flag = impl_flag(&name, &variants, self.unknown);
        let impl_flag_display = impl_flag_display(&name);

        let (return_type, default) = if self.unknown {
            (quote!(Self), quote!(Self::Unknown))
        } else {
            (quote!(Option<Self>), quote!(None))
        };

        let unknown_cb = self.unknown_callback.as_deref().unwrap_or("|_| ()");
        let unknown_cb = syn::parse_str::<syn::ExprClosure>(unknown_cb).unwrap();

        let mut new = Vec::new();
        for variant in variants {
            let ident = &variant.rust;
            let value = quote!(Self::#ident);
            let value = if self.unknown {
                value
            } else {
                quote!(Some(#value))
            };

            if variant.betaflight == variant.inav && variant.betaflight.is_some() {
                let index = variant.betaflight.unwrap();
                let arm = quote!(#index => #value);
                new.push((index, Firmware::Both, arm));
                continue;
            }

            if let Some(index) = variant.betaflight {
                let arm = quote!(#index if fw.is_betaflight() => #value);
                new.push((index, Firmware::Betaflight, arm));
            }

            if let Some(index) = variant.inav {
                let arm = quote!(#index if fw.is_inav() => #value);
                new.push((index, Firmware::Inav, arm));
            }
        }
        new.sort_unstable_by_key(|(index, firmware, _)| (*index, *firmware));
        let new = new.iter().map(|(_, _, arm)| arm);

        quote! {
            #attrs
            #enum_def
            #impl_flag
            #impl_flag_display

            #[allow(unused_qualifications, clippy::match_same_arms, clippy::unseparated_literal_suffix)]
            impl #name {
                pub(crate) fn new(raw: u32, fw: crate::headers::InternalFirmware) -> #return_type {
                    match raw {
                        #(#new,)*
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            (#unknown_cb)(raw);
                            #default
                        }
                    }
                }
            }
        }
    }
}
