mod either;

use proc_macro::TokenStream;

#[proc_macro]
pub fn either(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as either::Expr);
    ast.into_either_expr()
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}
