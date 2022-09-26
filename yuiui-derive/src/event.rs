use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens, TokenStreamExt as _};

pub struct EventDerive {
    name: syn::Ident,
    variants: Vec<(syn::Type, TokenStream2)>,
}

impl EventDerive {
    pub fn from_enum(item: syn::ItemEnum) -> Self {
        let name = item.ident;
        let mut variants = vec![];

        for variant in item.variants {
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

        Self { name, variants }
    }
}

impl ToTokens for EventDerive {
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
