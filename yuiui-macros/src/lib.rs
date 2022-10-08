mod either;
mod event;

use proc_macro::TokenStream;

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemEnum);
    event::derive_event_impl(&ast)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}

#[proc_macro]
pub fn either(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as either::Expr);
    ast.into_either_expr()
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}
