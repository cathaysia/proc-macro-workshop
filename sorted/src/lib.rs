use proc_macro::TokenStream;
use proc_macro2;
use syn::{
    DeriveInput, Item,
    __private::{quote::quote, ToTokens},
    parse_macro_input, Error,
};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let input = parse_macro_input!(input as DeriveInput);
    let input = Item::from(input);
    match input {
        Item::Enum(data) => data.to_token_stream().into(),
        _ => Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        )
        .to_compile_error()
        .into(),
    }
}
