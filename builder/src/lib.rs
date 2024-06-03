use std::collections::HashMap;

use proc_macro::TokenStream;
use syn::{DeriveInput, Expr, GenericArgument, Lit, Path, PathArguments, Type};
use quote::{format_ident, quote, TokenStreamExt};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = syn::parse(input).unwrap();
    let ident = derive_input.ident;
    let builder_name = format_ident!("{ident}Builder");
    let mut must_have_fields = Vec::new();
    let mut option_fields = Vec::new();
    let mut vec_fields = Vec::new();
    match derive_input.data {
        syn::Data::Struct(st) => {
            let mut builder_fields = quote!{};
            let mut initialize_fields = quote!{};
            let mut field_setters = quote!{};
            let mut each_fields = HashMap::new();
            // let mut must_have_fields = Vec::new();
            for field in st.fields.iter() {
                let field_ident = &field.ident;
                let field_ty = &field.ty;
                let mut stop = false;
                for attr in &field.attrs {
                    if attr.meta.path().is_ident("builder") {
                        let args: Expr = attr.parse_args().unwrap();
                        // eprintln!("{:#?}", args);
                        if let Expr::Assign(assign) = args {
                            let left = assign.left;
                            let right = assign.right;
                            if let Expr::Path(path) = *left {
                                if path.path.is_ident("each") {
                                    if let Expr::Lit(lit) = *right {
                                        if let Lit::Str(litstr) = lit.lit {
                                            let p = litstr.parse::<Path>().unwrap();
                                            let id = p.get_ident().unwrap();
                                            each_fields.insert(field_ident, (field_ty, id.clone()));
                                            stop = true;
                                            builder_fields.append_all(quote!{
                                                #field_ident: #field_ty,
                                            });
                                            if let Type::Path(type_path) = field_ty {
                                                let segment = &type_path.path.segments[0];
                                                // eprintln!("segments: {:#?}", segment);
                                                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                                                    let arg = &args.args[0];
                                                    if let GenericArgument::Type(Type::Path(pp)) = arg {
                                                        let inner_ty_ident = pp.path.get_ident();
                                                        field_setters.append_all(quote!{
                                                            fn #id(&mut self, ele: #inner_ty_ident) -> &mut Self {
                                                                self.#field_ident.push(ele);
                                                                self
                                                            }
                                                        });
                                                    }
                                                }

                                            }
                                            initialize_fields.append_all(quote!{
                                                #field_ident: Vec::new(),
                                            });
                                            vec_fields.push(field.ident.clone());
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if stop {
                    continue;
                }
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
                            #( #vec_fields: self.#vec_fields.clone(), )*
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
