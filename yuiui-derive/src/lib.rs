mod event;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::DeriveInput);
    let event_derive = match parsed.data {
        syn::Data::Struct(data) => event::EventDerive::from_struct(parsed.ident, data),
        syn::Data::Enum(data) => event::EventDerive::from_enum(parsed.ident, data),
        syn::Data::Union(_) => {
            panic!("Event implementations cannot be derived from union")
        }
    };
    let tokens = TokenStream::from(quote!(#event_derive));
    tokens
}
