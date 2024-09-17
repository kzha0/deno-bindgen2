
use deno_bindgen2::deno_bindgen;

#[deno_bindgen]
fn test(tup: u8, new: (u8, u8), some_string: &mut str) {
    let x = tup + 1;
    x;
}





fn test_str(some_str: &str) {}
fn test_slice(some_slice: &[u8]) {}
fn test_custom(some_type: CustomType) {}
fn test_string(some_string: String) {}



// #[derive(Clone, Debug)]
pub struct CustomType {}
impl CustomType {
    fn test_self(self) -> Self { self }
}

fn str_as_mut(arg_0: *mut u8, arg_1: usize) {
    let slice = unsafe { std::slice::from_raw_parts_mut(arg_0, arg_1) };
    let str = unsafe { std::str::from_utf8_unchecked_mut(slice) };

    test_str(&*str);
    test_slice(&*slice);
}


fn custom_ptr(arg_0: *mut ()) -> *mut () {
    // check lifetime
    // check validity of pointer
    let arg_0 = unsafe { Box::from_raw(arg_0 as _) };

    let ret = CustomType::test_self(*arg_0);

    Box::into_raw(Box::new(ret)) as *mut _
}

fn test_extern_naming(__arg_0: usize, __arg_1: *mut (), __arg_2: usize) {

}

// fn make_str(arg_0: *mut u8, arg_1: usize) {
//     let slice = unsafe { std::slice::from_raw_parts_mut(arg_0, arg_1) };
//     let string = String::from_utf8_unchecked(bytes)
// }