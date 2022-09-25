use proc_macro2::Literal;
use quote::{quote, ToTokens, TokenStreamExt as _};

pub struct WidgetBuilderDerive {
    widget_type: syn::Type,
    item: syn::ItemStruct,
}

impl WidgetBuilderDerive {
    pub fn new(widget_type: syn::Type, item: syn::ItemStruct) -> Self {
        Self { widget_type, item }
    }
}

impl ToTokens for WidgetBuilderDerive {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut new_arguments = Vec::new();
        let mut new_body = Vec::with_capacity(self.item.fields.len());
        let mut build_body = Vec::with_capacity(self.item.fields.len());
        let mut update_body = Vec::with_capacity(self.item.fields.len());
        let mut setter_fns = Vec::with_capacity(self.item.fields.len());

        for field in self.item.fields.iter() {
            let ty = &field.ty;
            let name = field
                .ident
                .as_ref()
                .expect("the field name must be specified");
            let property = Literal::string(&name.to_string().replace("_", "-"));

            if let Some(inner_ty) = extract_option(ty) {
                new_body.push(quote!(#name: None));

                build_body.push(quote!(
                    if let Some(ref #name) = self.#name {
                        properties.push((#property, #name));
                    }
                ));

                update_body.push(quote!(
                    match (&old.#name, &self.#name) {
                        (Some(old_value), Some(new_value)) => {
                            if old_value != new_value {
                                properties.push((#property, new_value.to_value()));
                            }
                        }
                        (Some(_), None) => {
                            let pspec = object.find_property(#property)
                                .expect(concat!("Unable to find the property of ", #property));
                            let default_value = pspec.default_value().to_value();
                            properties.push((#property, default_value));
                        }
                        (None, Some(new_value)) => {
                            properties.push((#property, new_value.to_value()));
                        }
                        (None, None) => {}
                    }
                ));

                setter_fns.push(quote!(
                    pub fn #name(mut self, #name: #inner_ty) -> Self {
                        self.#name = Some(#name);
                        self
                    }
                ));
            } else {
                new_arguments.push(quote!(#name: #ty));

                new_body.push(quote!(#name));

                build_body.push(quote!(
                    properties.push((#property, &self.#name));
                ));

                update_body.push(quote!(
                    if old.#name != self.#name {
                        properties.push((#property, self.#name.to_value()));
                    }
                ));
            }
        }

        let widget_type = &self.widget_type;
        let ident = &self.item.ident;
        let (impl_generics, ty_generics, where_clause) = self.item.generics.split_for_impl();

        tokens.append_all(quote! {
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
                        .expect(concat!("Failed to create an instance of ", stringify!(#widget_type)))
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
        });
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