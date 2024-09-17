use proc_macro::TokenStream;

mod r#mod;
use r#mod::*;

/*========================================================================*/

// For Developers

// `deno_bindgen2` is a bindings generator that simplifies porting of Rust libraries into Deno through its FFI API

// For a quick rundown of the overall code hierarchy,
// lib.rs -- this is the topmost file that exports the main `deno_bindgen` macro
// mod.rs -- routes other components and exposes internal modules to eachother, without exposing them to the public API

// This main macro retains its name/identifier from the original `deno_bindgen` crate for backwards compatibility and ease of switching, despite being an entirely unrelated crate.

// TODO: Rename this fn to `deno_bindgen2` and provide an alias for the old `deno_bindgen` attribute macro

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn deno_bindgen(attr: TokenStream,input: TokenStream) -> TokenStream {
    Parser::from_attr_stream(attr, input)
}
