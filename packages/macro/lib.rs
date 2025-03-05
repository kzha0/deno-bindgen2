use deno_bindgen2_common::Marker;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn deno_bindgen(_attr: TokenStream, input: TokenStream) -> TokenStream {
    Marker::deno_bindgen(input.into()).into()
}

#[proc_macro_attribute]
pub fn non_blocking(_attr: TokenStream, input: TokenStream) -> TokenStream {
    Marker::non_blocking(input)
}
