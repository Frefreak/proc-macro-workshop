use proc_macro::TokenStream;
use syn::{DeriveInput, Type};
use quote::{format_ident, quote, TokenStreamExt};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();
    let ident = derive_input.ident;
    let builder_name = format_ident!("{ident}Builder");
    let mut must_have_fields = Vec::new();
    let mut option_fields = Vec::new();
    match derive_input.data {
        syn::Data::Struct(st) => {
            let mut builder_fields = quote!{};
            let mut initialize_fields = quote!{};
            let mut field_setters = quote!{};
            // let mut must_have_fields = Vec::new();
            for field in st.fields.iter() {
                let field_ident = &field.ident;
                let field_ty = &field.ty;
                initialize_fields.append_all(quote!{
                    #field_ident: None,
                });
                if let Type::Path(path) = &field.ty {
                    let segs = &path.path.segments;
                    if !segs.is_empty() && segs[0].ident == format_ident!("Option") {
                        option_fields.push(field.ident.clone());
                        let inner_ty = match &segs[0].arguments {
                            syn::PathArguments::None => panic!("unexpected path None"),
                            syn::PathArguments::AngleBracketed(arg) => {
                                match &arg.args[0] {
                                    syn::GenericArgument::Type(ty) => {
                                        ty
                                    },
                                    _ => panic!("unexpected arg"),
                                }
                            }
                            syn::PathArguments::Parenthesized(_) => panic!("unexpected path Parenthesized"),
                        };
                        // eprintln!("{:#?}", segs[0]);
                        builder_fields.append_all(quote!{
                            #field_ident: #field_ty,
                        });
                        field_setters.append_all(quote!{
                            fn #field_ident(&mut self, #field_ident: #inner_ty) -> &mut Self {
                                self.#field_ident = Some(#field_ident);
                                self
                            }
                        });
                    } else {
                        must_have_fields.push(field.ident.clone());
                        builder_fields.append_all(quote!{
                            #field_ident: Option<#field_ty>,
                        });
                        field_setters.append_all(quote!{
                            fn #field_ident(&mut self, #field_ident: #field_ty) -> &mut Self {
                                self.#field_ident = Some(#field_ident);
                                self
                            }
                        });
                    }
                } else {
                    must_have_fields.push(field.ident.clone());
                    builder_fields.append_all(quote!{
                        #field_ident: Option<#field_ty>,
                    });
                    field_setters.append_all(quote!{
                        fn #field_ident(&mut self, #field_ident: #field_ty) -> &mut Self {
                            self.#field_ident = Some(#field_ident);
                            self
                        }
                    });
                }
                // eprintln!("{:#?}", field.ty);
            }
            let idents2 = must_have_fields.clone();
            let ts2 = quote! {
                pub struct #builder_name {
                    #builder_fields

                }
                impl #builder_name {
                    #field_setters

                    pub fn build(&mut self) -> Result<#ident, Box<dyn ::std::error::Error>> {
                        #( if self.#must_have_fields.is_none() {
                            return Err(format!("missing field: {}", stringify!(#must_have_fields)).into())
                        })*

                        return Ok(#ident {
                            #( #idents2: self.#idents2.clone().unwrap(), )*
                            #( #option_fields: self.#option_fields.clone(), )*
                        })
                    }
                }

                impl #ident {
                    pub fn builder() -> #builder_name {
                        #builder_name {
                            #initialize_fields
                        }
                    }
                }
            };
            // eprintln!("{:#?}", ts2);
            ts2.into()
        }
        _ => {
            panic!("only support struct");
        }
    }
}
