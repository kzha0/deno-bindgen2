#![feature(trace_macros)]
#![feature(str_from_raw_parts)]

use deno_bindgen2::deno_bindgen;

trace_macros!(true);

#[deno_bindgen]
fn test(tup: u8, new: (u8, u8), some_string: &str) {
    let x = tup + 1;
    x;
}
