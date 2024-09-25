
use deno_bindgen2::deno_bindgen;

#[deno_bindgen]
fn test(tup: u8, new: (u8, u8), some_string: &mut str) -> &str {
    let x = "some_str";
    x
}



#[derive(Clone, Debug, Default)]
pub struct CustomType {}
impl CustomType {
    fn test_self(self) -> Self { self }
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
    unsafe { (*arg_0).0  }
}