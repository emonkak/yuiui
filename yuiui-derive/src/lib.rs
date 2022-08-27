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
        let type_ids = self
            .variants
            .iter()
            .map(|(ty, _)| quote!(::std::any::TypeId::of::<#ty>()))
            .collect::<Vec<_>>();
        let downcasts = self
            .variants
            .iter()
            .map(|(ty, construct)| {
                quote!(
                    if value.type_id() == ::std::any::TypeId::of::<#ty>() {
                        let value = value.downcast_ref().unwrap();
                        return Some(#construct);
                    }
                )
            })
            .collect::<Vec<_>>();
        let transmutes = self
            .variants
            .iter()
            .map(|(ty, construct)| {
                quote!(
                    if ::std::any::TypeId::of::<T>() == ::std::any::TypeId::of::<#ty>() {
                        let value = unsafe { std::mem::transmute(value) };
                        return Some(#construct);
                    }
                )
            })
            .collect::<Vec<_>>();

        tokens.append_all(quote! {
            impl<'event> ::yuiui::Event<'event> for #name<'event> {
                fn allowed_types() -> Vec<::std::any::TypeId> {
                    vec![#(#type_ids),*]
                }

                fn from_any(value: &'event dyn ::std::any::Any) -> Option<Self> {
                    #(#downcasts)*
                    None
                }

                fn from_static<T: 'static>(value: &'event T) -> Option<Self> {
                    #(#transmutes)*
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
        let ty = unreference(field.ty);
        let construct = match &field.ident {
            Some(field_name) => quote!(#name { #field_name: value }),
            None => quote!(#name(value)),
        };
        variants.push((ty, construct));
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
            let ty = unreference(field.ty);
            let variant_name = &variant.ident;
            let construct = match &field.ident {
                Some(field_name) => {
                    quote!(#name::#variant_name { #field_name: value })
                }
                None => quote!(#name::#variant_name(value)),
            };
            variants.push((ty, construct));
        }
    }

    EventImpl { name, variants }
}

fn unreference(ty: syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(reference) => unreference(*reference.elem),
        _ => ty,
    }
}
