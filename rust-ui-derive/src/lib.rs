extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(WidgetMeta)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = parse_macro_input!(input);
    let name = ast.ident;
    let output = if !ast.generics.params.is_empty() {
        let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
        quote! {
            impl #impl_generics WidgetMeta for #name #ty_generics #where_clause {
                fn as_any(&self) -> &::std::any::Any {
                    self
                }
            }
        }
    } else {
        quote! {
            impl WidgetMeta for #name {
                fn as_any(&self) -> &::std::any::Any {
                    self
                }
            }
        }
    };
    output.into()
}
