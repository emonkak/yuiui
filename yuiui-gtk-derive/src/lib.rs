mod widget_builder;

use proc_macro::TokenStream;

#[proc_macro_derive(WidgetBuilder, attributes(property, widget))]
pub fn gtk_view(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemStruct);
    widget_builder::derive_widget_builder(&ast)
        .unwrap_or_else(|error| error.to_compile_error())
        .into()
}
