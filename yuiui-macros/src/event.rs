use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

pub(super) fn derive_event_impl(item: &syn::ItemEnum) -> syn::Result<TokenStream2> {
    let name = &item.ident;
    let mut types_sinature = quote!([::std::any::TypeId; 0]);
    let mut types_body = Vec::with_capacity(item.variants.len());
    let mut from_any_body = Vec::with_capacity(item.variants.len());

    for (i, variant) in item.variants.iter().enumerate() {
        if variant.fields.len() != 1 {
            Err(syn::Error::new(
                variant.fields.span(),
                "the enum variant must have only one field",
            ))?;
        }

        let field = variant.fields.iter().next().unwrap();
        let variant_name = &variant.ident;
        let ty = &field.ty;

        let construct = match &field.ident {
            Some(field_name) => {
                quote!(#name::#variant_name { #field_name: payload })
            }
            None => quote!(#name::#variant_name(payload)),
        };

        let lifetime = extract_lifetime(ty).ok_or_else(|| {
            syn::Error::new(field.span(), "the field type must be specified a lifetime")
        })?;

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
            if let Some(payload) = <#ty as yuiui::Event<#lifetime>>::from_any(payload) {
                return Some(#construct);
            }
        ));
    }

    Ok(quote! {
        impl<'event> yuiui::Event<'event> for #name<'event> {
            type Types = #types_sinature;

            fn types() -> Self::Types {
                #(#types_body)*
                iter
            }

            fn from_any(payload: &'event dyn ::std::any::Any) -> Option<Self> {
                #(#from_any_body)*
                None
            }
        }
    })
}

fn extract_lifetime(ty: &syn::Type) -> Option<&syn::Lifetime> {
    match ty {
        syn::Type::Reference(reference) => reference.lifetime.as_ref(),
        _ => None,
    }
}
