mod widget_builder;

use proc_macro::TokenStream;
use quote::quote;
use std::mem;

#[proc_macro_derive(WidgetBuilder, attributes(property, widget))]
pub fn gtk_view(input: TokenStream) -> TokenStream {
    let mut parsed = syn::parse_macro_input!(input as syn::ItemStruct);
    for attr in mem::take(&mut parsed.attrs) {
        if attr.path.is_ident("widget") {
            let widget: syn::Type = attr.parse_args().expect("invalid widget type");
            let widget_builder_derive = widget_builder::WidgetBuilderDerive::new(widget, parsed);
            let tokens = TokenStream::from(quote!(#widget_builder_derive));
            return tokens;
        }
    }
    panic!("the widget type must be specified by the #[widget(..)] attribute.");
}
