use proc_macro::TokenStream;
use syn::{
    DeriveInput, Item,
    __private::{quote::quote, ToTokens},
    parse_macro_input,
};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let input = parse_macro_input!(input as DeriveInput);
    let input = Item::from(input);
    if let Item::Enum(data) = input {
        return data.to_token_stream().into();
    }

    quote! {}.into()
}
