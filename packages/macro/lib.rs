use deno_bindgen2_common::Item;
use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn deno_bindgen(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut out = input.clone();

    let mut item = syn::parse_macro_input!(input as Item);
    item.to_ffi_safe();

    out.extend(&mut TokenStream::from(quote::ToTokens::into_token_stream(item)).into_iter());

    out
}

// #[proc_macro_attribute]
// pub fn test_attr(attr: TokenStream, _: TokenStream) -> TokenStream {
//     let attrs = parse_macro_input!(attr as MacroAttribute);
//     panic!("{attrs:#?}");
// }

// #[derive(Debug)]
// struct MacroAttribute {
//     meta: Meta
// }

// impl Parse for MacroAttribute {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         Ok(Self { meta: input.parse()? })
//     }
// }
