// note: use the `rust-analyzer: Expand macro recursively at caret` command to see the macro in action!

use deno_bindgen2::deno_bindgen;

#[deno_bindgen]
fn test(tup: u8, new: (u8, u8), some_string: &mut str, some_ref: &usize) -> &'static str {
    let x = "some_str";
    x
}

#[deno_bindgen]
fn test_complex() -> (usize, &'static str, CustomType) {
    let str = "asd";
    let out = (10, str, (CustomType {}));
    out
}

#[deno_bindgen]
fn test_slice(some_slice: &[u8]) -> &[u16] {
    let out: &[u16] = &[123, 12, 2];
    out
}

#[deno_bindgen]
fn test_ref_tup(tup: &(usize, &str, (usize, u32))) {}

#[deno_bindgen]
fn test_string(arg_0: String) -> String {
    arg_0
}

fn test_str() {
    let out = "str";
    let mut out: Box<str> = Box::from(out.to_owned());

    let out = (out.as_mut_ptr(), out.len());

    let out = Box::from(out);
    let out = Box::into_raw(out);
}

#[deno_bindgen]
fn test_custom() -> CustomType {
    let x = CustomType {};
    x
}

#[deno_bindgen]
fn test_tup() -> (usize, i32, i32) {
    let tup = (423, 5235, 2353);
    tup
}

fn get_str_ptr(arg_0: *mut (*mut u8, usize)) -> *mut u8 {
    unsafe { (*arg_0).0 }
}

fn drop_ptr(arg_0: *mut (*mut u8, usize)) {
    unsafe { drop(std::slice::from_raw_parts_mut((*arg_0).0, (*arg_0).1)) };
    unsafe { drop(Box::from_raw(arg_0)) };
}

#[derive(Clone, Debug, Default)]
pub struct CustomType {}

#[deno_bindgen]
impl CustomType {
    fn test_self(self) -> Self {
        self
    }
    fn unrelated(num: usize) {
        print!("something")
    }
}

// fn test_box(arg: *mut CustomType) {
//     let x = unsafe { *Box::from_raw(arg) };
// }

fn take_mut(arg_0: &str) {}


fn test_slice_3(slice: &[u32]) -> &[u32] {
    slice
}