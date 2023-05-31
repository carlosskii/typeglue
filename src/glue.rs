use proc_macro::TokenStream;
use proc_macro_error::{abort, emit_error};
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Data, Fields, GenericParam, Meta, Field};


pub fn struct_named_impl(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let data = match ast.data.clone() {
        Data::Struct(data) => data,
        _ => unreachable!()
    };

    let generics = generics.params.iter().map(|p| {
        if let GenericParam::Const(_) = p {
            abort!(p, "Glue cannot be derived for structs with const generics")
        }

        p
    }).collect::<Vec<_>>();

    let fields = data.fields.iter().collect::<Vec<_>>();
    let ignored_fields = get_ignored_fields(&fields);

    if ignored_fields.len() == fields.len() {
        abort!(name, "Glue structs cannot ignore all fields")
    }

    let fields = fields.iter().filter(|f| {
        !ignored_fields.contains(f)
    }).collect::<Vec<_>>();

    let ensure_default = if ignored_fields.is_empty() {
        quote! {}
    } else {
        quote_spanned! { name.span() =>
            struct _AssertDefault where #name: ::core::default::Default;
        }
    };

    let mut ending_defaults = ignored_fields.iter().map(|f| {
        let field_name = f.ident.as_ref().unwrap();

        quote! {
            #field_name: ::core::default::Default::default()
        }
    }).collect::<Vec<_>>();

    if !ending_defaults.is_empty() {
        ending_defaults.insert(0, quote! { , });
    }

    let ending_defaults = quote! {
        #(#ending_defaults)*
    };

    if fields.len() == 1 {
        let field = fields[0];
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;

        let generics = quote! {
            <#(#generics),*>
        };

        let gen = quote! {
            impl #generics From<#field_ty> for #name #generics {
                fn from(value: #field_ty) -> Self {
                    #ensure_default

                    Self {
                        #field_name: value
                        #ending_defaults
                    }
                }
            }

            impl #generics From<#name #generics> for #field_ty {
                fn from(value: #name #generics) -> Self {
                    value.#field_name
                }
            }
        };

        gen.into()
    } else {
        let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect::<Vec<_>>();
        let field_tys = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();

        let generics = quote! {
            <#(#generics),*>
        };

        let fields = field_names.iter()
            .enumerate()
            .map(|(i, name)| {
                let index = syn::Index::from(i);

                quote! {
                    #name: value.#index
                }
            }).collect::<Vec<_>>();
        
        let gen = quote! {
            impl #generics From<(#(#field_tys),*)> for #name #generics {
                fn from(value: (#(#field_tys),*)) -> Self {
                    #ensure_default

                    Self {
                        #(#fields),*
                        #ending_defaults
                    }
                }
            }
        };

        gen.into()

    }
}

pub fn struct_unnamed_impl(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let data = match ast.data.clone() {
        Data::Struct(data) => data,
        _ => unreachable!()
    };

    let generics = generics.params.iter().map(|p| {
        if let GenericParam::Const(_) = p {
            abort!(p, "Glue cannot be derived for structs with const generics")
        }

        p
    }).collect::<Vec<_>>();

    let fields = data.fields.iter().collect::<Vec<_>>();
    let ignored_fields = get_ignored_fields(&fields);

    if !ignored_fields.is_empty() {
        abort!(name, "Glue tuple structs cannot ignore fields")
    }

    if fields.len() == 1 {
        let field = fields[0];
        let field_ty = &field.ty;

        let generics = quote! {
            <#(#generics),*>
        };

        let gen = quote! {
            impl #generics From<#field_ty> for #name #generics {
                fn from(value: #field_ty) -> Self {
                    Self(value)
                }
            }

            impl #generics From<#name #generics> for #field_ty {
                fn from(value: #name #generics) -> Self {
                    value.0
                }
            }
        };

        gen.into()
    } else {
        let field_tys = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
        let field_indices = (0..field_tys.len()).map(syn::Index::from).collect::<Vec<_>>();

        let generics = quote! {
            <#(#generics),*>
        };

        let gen = quote! {
            impl #generics From<(#(#field_tys),*)> for #name #generics {
                fn from(value: (#(#field_tys),*)) -> Self {
                    Self(#(value.#field_indices),*)
                }
            }
        };

        gen.into()
    }
}

pub fn enum_impl(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let data = match ast.data.clone() {
        Data::Enum(data) => data,
        _ => unreachable!()
    };

    if !generics.params.is_empty() {
        abort!(ast, "Glue cannot be derived for generic enums")
    }

    let variants = data.variants
        .iter()
        .filter_map(|v| {
            let ident = &v.ident;
            let fields = match &v.fields {
                Fields::Named(fields) => {
                    emit_error!(fields, "Glue cannot be derived for enums with named fields");
                    return None
                },
                Fields::Unit => {
                    emit_error!(v, "Glue cannot be derived for enums with unit variants");
                    return None
                },
                Fields::Unnamed(fields) => {
                    fields
                }
            };

            if fields.unnamed.len() == 1 {
                let field = &fields.unnamed[0];
                let field_ty = &field.ty;

                Some(quote! {
                    impl From<#field_ty> for #name {
                        fn from(value: #field_ty) -> Self {
                            Self::#ident(value)
                        }
                    }
                })
            } else {
                let field_tys = fields.unnamed.iter().map(|f| &f.ty).collect::<Vec<_>>();
                let field_indices = (0..field_tys.len()).map(syn::Index::from).collect::<Vec<_>>();

                Some(quote! {
                    impl From<(#(#field_tys),*)> for #name {
                        fn from(value: (#(#field_tys),*)) -> Self {
                            Self::#ident(#(value.#field_indices),*)
                        }
                    }
                })
            }
        });
    
    let gen = quote! {
        #(#variants)*
    };

    gen.into()
}

fn get_ignored_fields<'a>(fields: &'a [&'a Field]) -> Vec<&'a Field> {
    fields.iter().filter(|f| {
        f.attrs.iter().any(|a| {
            if !a.path().is_ident("glue") {
                return false;
            }

            if let Ok(meta) = a.parse_args() {
                if let Meta::Path(path) = meta {
                    if path.is_ident("ignore") {
                        true
                    } else {
                        emit_error!(a, "Expected `#[glue(ignore)]`");
                        false
                    }
                } else {
                    emit_error!(a, "Expected `#[glue(ignore)]`");
                    false
                }
            } else {
                emit_error!(a, "Expected `#[glue(ignore)]`");
                false
            }
        })
    }).cloned().collect::<Vec<_>>()
}