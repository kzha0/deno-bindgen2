// benchmarks
#![feature(test)]
extern crate test;

use deno_bindgen2::*;

#[deno_bindgen]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use deno_bindgen2_common::Marker;
    use proc_macro2::TokenStream;
    use quote::quote;
    use test::Bencher;

    use super::*;

    // we measure proc-macro processing performance here
    #[bench]
    fn bench_add(b: &mut Bencher) {
        let input: TokenStream = quote! {
            pub fn add(left: u64, right: u64) -> u64 {
                left + right
            }
        };

        b.iter(|| {
            Marker::deno_bindgen(input.clone());
        });
    }
}
