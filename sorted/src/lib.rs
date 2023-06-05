use proc_macro::TokenStream;
use proc_macro2;
use syn::{
    DeriveInput, Item, __private::ToTokens, parse_macro_input, spanned::Spanned, Error, Variant,
};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let input = parse_macro_input!(input as DeriveInput);
    let input = Item::from(input);
    match input {
        Item::Enum(data) => {
            let source: Vec<_> = data
                .variants
                .iter()
                .map(|item| (item, item.ident.to_string()))
                .collect();
            let mut sorted = source.clone();
            sorted.sort_by(|a, b| a.1.cmp(&b.1));
            for (a, b) in source.iter().zip(sorted.iter()) {
                if a.1 != b.1 {
                    return Error::new(b.0.span(), format!("{} should sort before {}", b.1, a.1))
                        .to_compile_error()
                        .into();
                }
            }
            data.to_token_stream().into()
        }
        _ => Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        )
        .to_compile_error()
        .into(),
    }
}
