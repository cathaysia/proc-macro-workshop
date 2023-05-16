use proc_macro::TokenStream;
use quote::quote;
use syn::{
    self, spanned::Spanned, AngleBracketedGenericArguments, GenericArgument, Meta, MetaList, Path,
    PathSegment, TypePath,
};

fn get_option_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    ref args,
                    ..
                }) = seg.arguments
                {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

// Vec<String>
// 返回 String
fn get_inside_type(ty: &syn::Type) -> Option<&syn::Ident> {
    if let syn::Type::Path(syn::TypePath { ref path, .. }) = ty {
        for seg in &path.segments {
            if seg.ident.to_string() != "Vec" {
                continue;
            }
            if let syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                ref args,
                ..
            }) = &seg.arguments
            {
                if let GenericArgument::Type(syn::Type::Path(TypePath {
                    path: Path { segments, .. },
                    ..
                })) = args.last().unwrap()
                {
                    if let PathSegment { ref ident, .. } = segments.first().unwrap() {
                        return Some(&ident);
                    }
                }
            }
        }
    }
    None
}

// #[builder(each = "arg")]
// 返回 arg
fn get_user_specified_ident_for_vec(field: &syn::Field) -> syn::Result<Option<syn::Ident>> {
    for attr in &field.attrs {
        if !attr.path().is_ident("builder") {
            continue;
        }

        if let Meta::List(MetaList { tokens, .. }) = &attr.meta {
            let val = tokens.to_string();
            let mut val = val.split('=');
            match val.next() {
                Some(v) => {
                    let v = v.trim();
                    if v != "each" {
                        if let syn::Meta::List(ref list) = attr.meta {
                            return Err(syn::Error::new_spanned(
                                list,
                                r#"expected `builder(each = "...")`"#,
                            ));
                        }
                    }
                }
                None => {}
            }

            match val.last() {
                Some(val) => {
                    let b = val.replace("\"", "");
                    let b = b.trim();
                    return Ok(Some(syn::Ident::new(&b, tokens.span())));
                }
                None => {
                    return Ok(None);
                }
            }
        }
    }
    Ok(None)
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);

    let ident = st.ident.to_string();
    let builder_name = format!("{}Builder", ident);
    let builder_ident = syn::Ident::new(&builder_name, st.span());
    let struct_ident = st.ident;

    let syn::Data::Struct(fields) = st.data else{
        panic!("")
    };

    let mut field_ast = quote!();
    let mut field_init = quote!();
    let mut field_setter = quote!();
    let mut field_cond = quote!();
    let mut field_res = quote!();

    for (_, f) in fields.fields.iter().enumerate() {
        let (field_id, field_ty) = (&f.ident, &f.ty);
        let vec_type_ident = match get_user_specified_ident_for_vec(f) {
            Ok(v) => v,
            Err(err) => {
                return err.to_compile_error().into();
            }
        };
        let field_id = match field_id {
            Some(v) => v,
            None => {
                return Default::default();
            }
        };
        let inside = get_inside_type(field_ty);

        match get_option_type(field_ty) {
            Some(ty) => {
                field_ast.extend(quote! {
                    pub #field_id: Option<#ty>,
                });
                field_init.extend(quote! {
                    #field_id: None,
                });

                field_setter.extend(quote! {
                    fn #field_id(&mut self, #field_id: #ty) ->&mut Self{
                        self.#field_id = Some(#field_id);
                        self
                    }
                });

                field_res.extend(quote! {
                    #field_id:self.#field_id.clone(),
                });
            }
            None => {
                // 不是 Option
                field_ast.extend(quote! {
                    pub #field_id: #field_ty,
                });
                field_init.extend(quote! {
                    #field_id: Default::default(),
                });
                match vec_type_ident {
                    Some(ident) => {
                        let b = inside.unwrap();
                        if ident.to_string() != field_id.to_string() {
                            field_setter.extend(quote! {
                                fn #ident(&mut self, #field_id: #b) ->&mut Self{
                                    self.#field_id.push(#field_id);
                                    self
                                }
                            });
                        } else {
                            field_setter.extend(quote! {
                                fn #field_id(&mut self, #field_id: #b) ->&mut Self{
                                    self.#field_id.push(#field_id);
                                    self
                                }
                            });
                        }
                        let errmsg = format!("{} field is missing!", field_id.to_string());
                        if let None = inside {
                            field_cond.extend(quote! {
                                if self.#field_id.is_none() {
                                    return std::result::Result::Err(String::from(#errmsg).into());
                                }
                            });
                        }
                        field_res.extend(quote! {
                            #field_id: self.#field_id.clone(),
                        });
                    }
                    // 没有 vec 注释
                    None => {
                        field_setter.extend(quote! {
                            fn #field_id(&mut self, #field_id: #field_ty) ->&mut Self{
                                self.#field_id =#field_id;
                                self
                            }
                        });
                        field_res.extend(quote! {
                            #field_id: self.#field_id.clone(),
                        });
                    }
                }
            }
        }
    }

    let res = quote! {
        pub struct #builder_ident {
            #field_ast
        }

        impl #struct_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #field_init
                }
            }

        }

        impl #builder_ident {
            #field_setter


            pub fn build(&mut self) -> std::result::Result<#struct_ident, std::boxed::Box<dyn std::error::Error>> {
                #field_cond

                Ok(#struct_ident{
                    #field_res
                })
            }
        }
    };

    res.into()
}
