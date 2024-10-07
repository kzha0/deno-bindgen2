pub use deno_bindgen2_macro::deno_bindgen;
pub use linkme;
pub use linkme::distributed_slice;

mod generate;
mod ir;
pub use generate::*;
pub use ir::*;

#[distributed_slice]
pub static RAW_ITEMS: [RawItem];

#[no_mangle]
fn __deno_bindgen2_init(opt: Options) {
    generate(&RAW_ITEMS, opt).unwrap();
}