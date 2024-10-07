use deno_bindgen2::*;

// Organization of tests:
// Supercluster A: Bare functions
// Supercluster B: Associated functions (method implements)

// Cluster A: Outputs only
// Cluster B: Inputs only
// Cluster C: Mix of both
// Cluster D: Complex nesting

// Subcluster tests:
// - Trivial types
// - String types
// - Slice types
// - Paren types
// - Tuple types
// - Custom types

// Utility tests:
// Test for memory leaks (critical measure)
// Test for performance overhead (pure rust vs rust in deno performance in time complexity, cpu cycles, cpu time)
// Test for memory limits (stack overflows, nesting practical limits)



//*-------------------------------- Supercluster A: Bare functions ------------------------------*/

// MARK: OUTPUT ONLY

#[deno_bindgen]
fn sc_a_bool_out() -> bool {
    true
}
#[deno_bindgen]
fn sc_a_u8_out() -> u8 {
    255
}
#[deno_bindgen]
fn sc_a_u16_out() -> u16 {
    65535
}
#[deno_bindgen]
fn sc_a_u32_out() -> u32 {
    4294967295
}
#[deno_bindgen]
fn sc_a_u64_out() -> u64 {
    18446744073709551615
}
#[deno_bindgen]
fn sc_a_usize_out() -> usize {
    18446744073709551615
}
#[deno_bindgen]
fn sc_a_i8_out() -> i8 {
    -128
}
#[deno_bindgen]
fn sc_a_i16_out() -> i16 {
    -31872
}
#[deno_bindgen]
fn sc_a_i32_out() -> i32 {
    -2147483648
}
#[deno_bindgen]
fn sc_a_i64_out() -> i64 {
    -9223372036854775808
}
#[deno_bindgen]
fn sc_a_isize_out() -> isize {
    -9223372036854775808
}
#[deno_bindgen]
fn sc_a_f32_out() -> f32 {
    (2 as f32 /3 as f32) as f32
}
#[deno_bindgen]
fn sc_a_f64_out() -> f64 {
    (2 as f64 /3 as f64) as f64
}
#[deno_bindgen]
fn sc_a_pointer_out() -> *mut bool {
    Box::into_raw(Box::new(true))
}
#[deno_bindgen]
fn sc_a_str_out() -> &'static str {
    "Hello world!"
}
#[deno_bindgen]
fn sc_a_slice_out() -> &'static [u8] {
    &[255]
}
#[allow(unused_parens)]
#[deno_bindgen]
fn sc_a_paren_out() -> (bool) {
    (true)
}
#[deno_bindgen]
fn sc_a_tuple_out() -> (bool, usize, isize) {
    (true, 18446744073709551615, -9223372036854775808)
}

// MARK: INPUT ONLY

#[deno_bindgen]
fn sc_a_bool(arg_0: bool) {
    println!("sc_a_bool: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_u8(arg_0: u8) {
    println!("sc_a_u8: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_u16(arg_0: u16) {
    println!("sc_a_u16: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_u32(arg_0: u32) {
    println!("sc_a_u32: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_u64(arg_0: u64) {
    println!("sc_a_u64: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_usize(arg_0: usize) {
    println!("sc_a_usize: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_i8(arg_0: i8) {
    println!("sc_a_i8: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_i16(arg_0: i16) {
    println!("sc_a_i16: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_i32(arg_0: i32) {
    println!("sc_a_i32: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_i64(arg_0: i64) {
    println!("sc_a_i64: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_isize(arg_0: isize) {
    println!("sc_a_isize: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_f32(arg_0: f32) {
    println!("sc_a_f32: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_f64(arg_0: f64) {
    println!("sc_a_f64: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_pointer(arg_0: *mut bool) {
    let arg_0 = unsafe { *arg_0 };
    println!("sc_a_pointer: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_str(arg_0: &str) {
    println!("sc_a_str: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_slice(arg_0: &[u8]) {
    println!("sc_a_slice: {:#?}", arg_0);
}
#[allow(unused_parens)]
#[deno_bindgen]
fn sc_a_paren(arg_0: (bool)) {
    println!("sc_a_paren: {:#?}", arg_0);
}
#[deno_bindgen]
fn sc_a_tuple(arg_0: (bool, usize, isize)) {
    println!("sc_a_tuple: {:#?}", arg_0);
}

#[deno_bindgen]
fn test_1() {
    println!("Hello, world!");
}

#[deno_bindgen]
fn test_2(str: &str) -> String {
    format!("{} to Rust!", str)
}
