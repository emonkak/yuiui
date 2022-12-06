use bae::FromAttributes;
use proc_macro2::{Literal, TokenStream as TokenStream2};
use quote::quote;
use syn::spanned::Spanned;

pub(super) fn derive_widget_builder(item: &syn::ItemStruct) -> syn::Result<TokenStream2> {
    let widget_type: syn::Type = item
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path.is_ident("widget") {
                Some(attr)
            } else {
                None
            }
        })
        .map_or_else(
            || {
                Err(syn::Error::new(
                    item.span(),
                    "the widget type must be specified by the #[widget(..)] attribute",
                ))
            },
            |attr| attr.parse_args(),
        )?;

    let mut new_arguments = Vec::new();
    let mut new_body = Vec::with_capacity(item.fields.len());
    let mut build_body = Vec::with_capacity(item.fields.len());
    let mut update_body = Vec::with_capacity(item.fields.len());
    let mut setter_fns = Vec::with_capacity(item.fields.len());

    for field in &item.fields {
        let ty = &field.ty;
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(field.span(), "the field name must be specified"))?;
        let property = Property::from_attributes(&field.attrs).unwrap_or_default();
        let property_name = &property
            .name_literal()
            .unwrap_or_else(|| Literal::string(&field_name.to_string().replace("_", "-")));

        if property.enables_argument() {
            new_arguments.push(quote!(#field_name: #ty));
            new_body.push(quote!(#field_name));
        } else {
            new_body.push(quote!(#field_name: Default::default()));
        }

        if let Some(inner_ty) = extract_option(ty) {
            if property.enables_bind() {
                build_body.push(quote!(
                    if let Some(ref #field_name) = self.#field_name {
                        properties.push((#property_name, #field_name));
                    }
                ));

                update_body.push(quote!(
                    match (&old.#field_name, &self.#field_name) {
                        (Some(old_value), Some(new_value)) => {
                            if old_value != new_value {
                                properties.push((#property_name, new_value.to_value()));
                            }
                        }
                        (Some(_), None) => {
                            let pspec = object.find_property(#property_name)
                                .expect(concat!("unable to find the property of ", #property_name));
                            let default_value = pspec.default_value().to_value();
                            properties.push((#property_name, default_value));
                        }
                        (None, Some(new_value)) => {
                            properties.push((#property_name, new_value.to_value()));
                        }
                        (None, None) => {}
                    }
                ));
            }

            if property.enables_setter() {
                setter_fns.push(quote!(
                    pub fn #field_name(mut self, #field_name: #inner_ty) -> Self {
                        self.#field_name = Some(#field_name);
                        self
                    }
                ));
            }
        } else {
            if property.enables_bind() {
                build_body.push(quote!(
                    properties.push((#property_name, &self.#field_name));
                ));

                update_body.push(quote!(
                    if old.#field_name != self.#field_name {
                        properties.push((#property_name, self.#field_name.to_value()));
                    }
                ));
            }

            if property.enables_setter() {
                setter_fns.push(quote!(
                    pub fn #field_name(mut self, #field_name: #ty) -> Self {
                        self.#field_name = #field_name;
                        self
                    }
                ));
            }
        }
    }

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            pub fn new(#(#new_arguments),*) -> Self {
                Self {
                    #(#new_body),*
                }
            }

            pub fn build(&self) -> #widget_type {
                let mut properties: Vec<(&str, &dyn glib::ToValue)> = vec![];
                #(#build_body)*
                glib::Object::new::<#widget_type>(&properties)
                    .expect(concat!("failed to create an instance of ", stringify!(#widget_type)))
            }

            pub fn force_update(&self, object: &#widget_type) {
                let mut properties: Vec<(&str, &dyn glib::ToValue)> = vec![];
                #(#build_body)*
                if !properties.is_empty() {
                    object.set_properties(&properties);
                }
            }

            pub fn update(&self, old: &Self, object: &#widget_type) -> bool {
                use glib::object::ObjectExt;
                use glib::value::ToValue;
                let mut properties: Vec<(&str, glib::Value)> = vec![];
                #(#update_body)*
                if !properties.is_empty() {
                    object.set_properties_from_value(&properties);
                    true
                } else {
                    false
                }
            }

            #(#setter_fns)*
        }
    })
}

#[derive(Default, FromAttributes)]
struct Property {
    bind: Option<syn::LitBool>,
    argument: Option<syn::LitBool>,
    setter: Option<syn::LitBool>,
    name: Option<syn::LitStr>,
}

impl Property {
    fn enables_bind(&self) -> bool {
        self.bind.as_ref().map_or(true, |lit| lit.value())
    }

    fn enables_argument(&self) -> bool {
        self.argument.as_ref().map_or(false, |lit| lit.value())
    }

    fn enables_setter(&self) -> bool {
        self.setter.as_ref().map_or(true, |lit| lit.value())
    }

    fn name_literal(&self) -> Option<Literal> {
        self.name.as_ref().map(|name| name.token())
    }
}

fn extract_option(ty: &syn::Type) -> Option<&syn::Type> {
    const OPTION_TYPES: [&[&str]; 3] = [
        &["Option"],
        &["std", "option", "Option"],
        &["core", "option", "Option"],
    ];

    if let syn::Type::Path(typepath) = ty {
        let segment_idents = typepath
            .path
            .segments
            .iter()
            .map(|segment| &segment.ident)
            .collect::<Vec<_>>();
        let is_option = OPTION_TYPES
            .iter()
            .any(|option_type| segment_idents.as_slice() == *option_type);
        if is_option {
            return typepath
                .path
                .segments
                .last()
                .and_then(|segment| match &segment.arguments {
                    syn::PathArguments::AngleBracketed(bracketed) => bracketed.args.first(),
                    _ => None,
                })
                .and_then(|arg| match arg {
                    syn::GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                });
        }
    }
    None
}
