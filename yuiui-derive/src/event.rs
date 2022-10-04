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
        let mut types_sinature = quote!([::std::any::TypeId; 0]);
        let mut types_body = Vec::with_capacity(self.variants.len());
        let mut from_any_body = Vec::with_capacity(self.variants.len());

        for (i, (ty, construct)) in self.variants.iter().enumerate() {
            let lifetime = extract_lifetime(ty).expect("extract lifetime");

            if i == 0 {
                types_sinature = quote!(
                    <<#ty as yuiui::Event<#lifetime>>::Types as IntoIterator>::IntoIter
                );
                types_body.push(quote!(
                    let iter = <#ty as yuiui::Event<#lifetime>>::types().into_iter();
                ));
            } else {
                types_sinature = quote!(
                    ::std::iter::Chain<
                        #types_sinature,
                        <<#ty as yuiui::Event<#lifetime>>::Types as IntoIterator>::IntoIter
                    >
                );
                types_body.push(quote!(
                    let iter = iter.chain(<#ty as yuiui::Event<#lifetime>>::types());
                ));
            }
            from_any_body.push(quote!(
                if let Some(event) = <#ty as yuiui::Event<#lifetime>>::from_any(event) {
                    return Some(#construct);
                }
            ));
        }

        tokens.append_all(quote! {
            impl<'event> yuiui::Event<'event> for #name<'event> {
                type Types = #types_sinature;

                fn types() -> Self::Types {
                    #(#types_body)*
                    iter
                }

                fn from_any(event: &'event dyn ::std::any::Any) -> Option<Self> {
                    #(#from_any_body)*
                    None
                }
            }
        });
    }
}

fn extract_lifetime(ty: &syn::Type) -> Option<&syn::Lifetime> {
    match ty {
        syn::Type::Reference(reference) => reference.lifetime.as_ref(),
        _ => None,
    }
}
