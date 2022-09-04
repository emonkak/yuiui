use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt as _};

#[proc_macro_derive(Event)]
pub fn event(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let event_impl = match ast.data {
        syn::Data::Struct(data) => impl_from_struct(ast.ident, data),
        syn::Data::Enum(data) => impl_from_enum(ast.ident, data),
        syn::Data::Union(_) => {
            panic!("Event implementations cannot be derived from union")
        }
    };
    let tokens = TokenStream::from(quote!(#event_impl));
    tokens
}

struct EventImpl {
    name: syn::Ident,
    variants: Vec<(syn::Type, TokenStream2)>,
}

impl ToTokens for EventImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let collect_types = self
            .variants
            .iter()
            .map(|(ty, _)| {
                quote!(
                    <#ty as ::yuiui::Event>::collect_types(type_ids);
                )
            })
            .collect::<Vec<_>>();
        let from_any = self
            .variants
            .iter()
            .map(|(ty, construct)| {
                quote!(
                    if let Some(event) = <#ty as ::yuiui::Event>::from_any(event) {
                        return Some(#construct);
                    }
                )
            })
            .collect::<Vec<_>>();

        tokens.append_all(quote! {
            impl<'event> ::yuiui::Event<'event> for #name<'event> {
                fn collect_types(type_ids: &mut Vec<::std::any::TypeId>) {
                    #(#collect_types)*
                }

                fn from_any(event: &'event dyn ::std::any::Any) -> Option<Self> {
                    #(#from_any)*
                    None
                }
            }
        });
    }
}

fn impl_from_struct(name: syn::Ident, data: syn::DataStruct) -> EventImpl {
    let mut variants = vec![];

    assert_eq!(data.fields.len(), 1, "struct must have only one field");

    for field in data.fields {
        let construct = match &field.ident {
            Some(field_name) => quote!(#name { #field_name: event }),
            None => quote!(#name(event)),
        };
        variants.push((field.ty, construct));
    }

    EventImpl { name, variants }
}

fn impl_from_enum(name: syn::Ident, data: syn::DataEnum) -> EventImpl {
    let mut variants = vec![];

    for variant in data.variants {
        assert_eq!(
            variant.fields.len(),
            1,
            "enum variant must have only one field"
        );

        for field in variant.fields {
            let variant_name = &variant.ident;
            let construct = match &field.ident {
                Some(field_name) => {
                    quote!(#name::#variant_name { #field_name: event })
                }
                None => quote!(#name::#variant_name(event)),
            };
            variants.push((field.ty, construct));
        }
    }

    EventImpl { name, variants }
}
