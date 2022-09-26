mod event;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let parsed = syn::parse_macro_input!(input as syn::ItemEnum);
    let event_derive = event::EventDerive::from_enum(parsed);
    let tokens = TokenStream::from(quote!(#event_derive));
    tokens
}
