extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(WidgetMeta)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);
    let output = quote! {
        impl WidgetMeta for #ident {
            fn as_any(&self) -> &::std::any::Any {
                self
            }
        }
    };
    output.into()
}
