use proc_macro::TokenStream;
use quote::quote;
use syn::{self, spanned::Spanned};

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

#[proc_macro_derive(Builder)]
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

        if field_id.is_none() {
        } else {
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
                        #field_id: Some(self.#field_id.clone().unwrap()),
                    });
                }
                None => {
                    field_ast.extend(quote! {
                        pub #field_id: Option<#field_ty>,
                    });
                    field_init.extend(quote! {
                        #field_id: None,
                    });
                    field_setter.extend(quote! {
                        fn #field_id(&mut self, #field_id: #field_ty) ->&mut Self{
                            self.#field_id = Some(#field_id);
                            self
                        }
                    });
                    field_cond.extend(quote! {
                        if self.#field_id.is_none() {
                            // let err = format!("{} field missing", &stringify!(#ident));
                            return Result::Err(String::new().into());
                        }
                    });
                    field_res.extend(quote! {
                        #field_id: self.#field_id.clone().unwrap(),
                    });
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


            pub fn build(&mut self) -> Result<#struct_ident, Box<dyn std::error::Error>> {
                #field_cond

                Ok(#struct_ident{
                    #field_res
                })
            }
        }
    };

    res.into()
}
