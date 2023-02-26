use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use serde::Deserialize;

use super::{get_as_name_arms, impl_flag, quote_attrs, str_to_ident, AugmentedData};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enum {
    name: String,
    doc: String,
    #[serde(default)]
    attrs: Vec<String>,
    unknown: bool,
    unknown_callback: Option<String>,
    #[serde(default)]
    rename: HashMap<String, String>,
    #[serde(default)]
    pub(super) data: HashMap<String, HashMap<u32, Vec<String>>>,
}

impl Enum {
    pub fn expand(&self) -> TokenStream {
        let name = self.name();
        let attrs = quote_attrs(&self.doc, &self.attrs, true);

        let (data, variants) = super::augment_data(&self.data, &self.rename);
        let new = self.expand_new(&data);

        let enum_def = variants
            .iter()
            .map(|(official, rust)| {
                let doc = format!("`{official}`");
                quote!(#[doc = #doc] #rust)
            })
            .chain(self.unknown.then_some(quote!(Unknown)));

        let as_name = get_as_name_arms(&variants)
            .chain(self.unknown.then_some(quote!(Self::Unknown => "UNKNOWN")));
        let impl_flag = impl_flag(&name, as_name);

        quote! {
            #attrs
            #[non_exhaustive]
            pub enum #name {
                #(#enum_def),*
            }

            #impl_flag
            #new
        }
    }

    fn name(&self) -> Ident {
        str_to_ident(&self.name)
    }

    fn expand_new(&self, data: &AugmentedData<'_>) -> TokenStream {
        let name = self.name();

        let mut arms = Vec::new();
        for ((_, rust), indices) in data {
            for (index, firmwares) in indices.iter() {
                let key = (index, firmwares[0].clone());
                let variant = quote!(Self::#rust);
                let variant = if self.unknown {
                    variant
                } else {
                    quote!(Some(#variant))
                };

                arms.push((key, quote!((#index, #(#firmwares)|*) => #variant)));
            }
        }
        arms.sort_unstable_by_key(|(key, _)| key.clone());
        let arms = arms.into_iter().map(|(_, tokens)| tokens);

        let unknown_callback = self.unknown_callback.as_deref().unwrap_or("|_| {}");
        let unknown_callback = syn::parse_str::<syn::ExprClosure>(unknown_callback).unwrap();

        let (unknown, return_type) = if self.unknown {
            (quote!(Self::Unknown), quote!(Self))
        } else {
            (quote!(None), quote!(Option<Self>))
        };

        quote! {
            #[allow(unused_qualifications, clippy::enum_glob_use, clippy::match_same_arms, clippy::unseparated_literal_suffix)]
            impl #name {
                pub(crate) fn new(raw: u32, fw: crate::headers::InternalFirmware) -> #return_type {
                    use crate::headers::InternalFirmware::*;
                    match (raw, fw) {
                        #(#arms,)*
                        _ => {
                            #[allow(clippy::redundant_closure_call)]
                            (#unknown_callback)(raw);
                            #unknown
                        }
                    }
                }
            }
        }
    }
}
