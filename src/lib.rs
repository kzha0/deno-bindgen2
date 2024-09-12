mod r#mod;
use proc_macro::TokenStream;
use proc_macro2::{
    Ident,
    Span,
};
use r#mod::*;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    FnArg,
    ItemFn,
};

/*========================================================================*/

// For Developers

// `deno_bindgen2` is a bindings generator that simplifies porting of Rust libraries into Deno through its FFI API

// For a quick rundown of the overall code hierarchy,
// lib.rs -- this is the topmost file that exports the main `deno_bindgen` macro
// mod.rs -- routes other components and exposes internal modules to eachother, without exposing them to the public API

// This main macro retains its name/identifier from the original `deno_bindgen` crate for backwards compatibility and ease of switching, despite being an entirely unrelated crate.

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn deno_bindgen(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    Parser::from_attr_stream(attr, input)
        .parse()
        .transform()
}
