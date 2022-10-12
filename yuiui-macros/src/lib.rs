mod either;

use proc_macro::TokenStream;
use syn::parse::Parser;

#[proc_macro]
pub fn either(input: TokenStream) -> TokenStream {
    either::parser
        .parse(input)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}
