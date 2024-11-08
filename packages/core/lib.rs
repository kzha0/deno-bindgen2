#![allow(unused_imports)]

pub use deno_bindgen2_macro::*;
pub use deno_bindgen2_utils::*;

/// Trait to let the tool identify a user-defined type/struct.
/// Used by the macro to auto-generate a drop implementation for a struct
/// When overriding this trait, make sure to provide a drop implementation
pub trait DenoBindgen {}

#[no_mangle]
pub static DENO_BINDGEN_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[no_mangle]
pub static DENO_BINDGEN_PKG_NAME: &str = env!("CARGO_PKG_NAME");
