
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

fn test_custom() {
    let x = CustomType {};
    let out = &x;
    let out = Box::from(out.to_owned());

    let out: Box<CustomType> = Box::from(out);
    let out = Box::into_raw(out);

}

fn test_tup() {
    let tup = (423, 5235, 2353);

    let mut out_tup = tup;
    let out = out_tup.0;
    let out = out as usize;
    out_tup.0 = out;
    let out = out_tup;
}


fn get_str_ptr(arg_0: *mut (*mut u8, usize)) -> *mut u8 {
    unsafe { (*arg_0).0  }
}