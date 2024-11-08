#![feature(str_from_raw_parts)]
#![allow(unexpected_cfgs)]

use deno_bindgen2_macro::deno_bindgen;

mod deno_bindgen2 {
    #[allow(dead_code)]
    pub(crate) trait DenoBindgen {}
}

struct Metadata;
impl deno_bindgen2::DenoBindgen for Metadata {}
#[deno_bindgen]
impl Metadata {
    fn rust_version() -> *const u8 {
        concat!(env!("CARGO_PKG_RUST_VERSION"), "\0").as_ptr()
    }
    fn rust_toolchain() -> *const u8 {
        concat!(env!("RUSTUP_TOOLCHAIN"), "\0").as_ptr()
    }
    fn lib_name() -> *const u8 {
        concat!(env!("CARGO_CRATE_NAME"), "\0").as_ptr()
    }
    fn lib_version() -> *const u8 {
        concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
    }
}

#[allow(dead_code)]
struct RustString;
impl deno_bindgen2::DenoBindgen for RustString {}
#[deno_bindgen]
#[cfg(deno_bindgen_rust_string)]
impl RustString {
    pub fn new() -> String {
        String::new()
    }
    pub fn from(ptr: *const u8, len: usize) -> String {
        unsafe { std::str::from_raw_parts(ptr, len).to_string() }
    }
    pub fn into_ptr(string: &String) -> *const u8 {
        string.as_ptr()
    }
    pub fn into_len(string: &String) -> usize {
        string.len()
    }
    pub fn push(string: &mut String, ptr: *const u8, len: usize) {
        string.push_str(unsafe { std::str::from_raw_parts(ptr, len) });
    }
    pub fn drop(string: String) {
        std::mem::drop(string);
    }
}
