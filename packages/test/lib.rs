pub use deno_bindgen2::*;

#[deno_bindgen]
fn test_1() {
    println!("Hello, world!");
}

#[deno_bindgen]
fn test_2(string: String) -> String {
    format!("{} to Rust!", string)
}

#[deno_bindgen]
fn test_empty() {}

#[deno_bindgen]
fn test_unit(_arg0: ()) -> () {}

#[deno_bindgen]
fn test_numeric(arg0: f64) -> f64 {
    arg0
}

#[deno_bindgen]
fn test_bool(arg0: bool) -> bool {
    !arg0 && true
}

#[deno_bindgen]
fn test_char(arg0: char) -> char {
    arg0
}

#[deno_bindgen]
fn test_ptr(arg0: *const u8) -> *const u8 {
    arg0
}

#[deno_bindgen]
fn test_ptr_mut(arg0: *mut u8) -> *mut u8 {
    arg0
}

#[deno_bindgen]
fn test_ref(arg0: &u8) -> &u8 {
    arg0
}


#[deno_bindgen]
fn test_mut(arg0: &mut u8) -> &mut u8 {
    arg0
}

#[deno_bindgen]
fn test_box(arg0: Box<u8>) -> Box<u8> {
    arg0
}

#[deno_bindgen]
fn test_fn_ptr(arg0: extern fn(u8) -> u8) -> extern fn(u8) -> u8 {
    arg0
}

#[deno_bindgen]
fn test_str(arg0: &str) -> &str {
    arg0
}

#[deno_bindgen]
fn test_string(arg0: String) -> String {
    arg0
}

#[deno_bindgen]
fn test_slice(arg0: &mut [u8]) -> &mut [u8] {
    arg0
}

#[deno_bindgen]
fn test_array(arg0: [u8; 8]) -> [u8; 8] {
    arg0
}

#[deno_bindgen]
fn test_vec(arg0: Vec<u8>) -> Vec<u8> {
    arg0
}

// [!TODO] provide way to supress these kinds of warnings
#[deno_bindgen]
fn test_path(arg0: std::string::String) -> std::string::String {
    arg0
}

#[deno_bindgen]
struct CustomType {}

#[deno_bindgen]
impl CustomType {

    unsafe fn test_self(self) -> Self {
        self
    }

    fn test_ref_self(&self) -> &Self {
        self
    }

    fn test_mut_self(&mut self) -> &mut Self {
        self
    }

    fn test_other_self(&self, _arg1: Self, _arg2: &mut CustomType) -> &Self {
        self
    }

    fn test_other_type(_arg0: SomeOtherType) {}
}

struct SomeOtherType {}
