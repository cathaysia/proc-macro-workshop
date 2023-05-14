use proc_macro::TokenStream;
use quote::quote;
use syn::{self, spanned::Spanned};

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

    for (_, f) in fields.fields.iter().enumerate() {
        let (field_id, field_ty) = (&f.ident, &f.ty);

        if field_id.is_none() {
        } else {
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
        }
    };
    // res.extend(quote! {
    //     impl #ident -> Self {
    //         pub fn builder() -> Self {
    //             Self {
    //
    //             }
    //         }
    //     }
    // });

    res.into()
}
