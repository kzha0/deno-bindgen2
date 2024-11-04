pub use deno_bindgen2_macro::*;

pub trait DenoBindgen {
    fn drop(self_: *mut Self) {
        drop(Box::from(self_));
    }
}

#[no_mangle]
pub static DENO_BINDGEN_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[no_mangle]
pub static DENO_BINDGEN_PKG_NAME: &str = env!("CARGO_PKG_NAME");
