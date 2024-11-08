import { test_bool, test_2 } from "./dist/libdeno_bindgen2_test.ts";
import { RustString } from "./dist/rust_type.ts";

Deno.test("test_bool", () => {
    console.log("this will be true: ", test_bool(false));
})


Deno.test("test_string", () => {
    const hello_string = test_2(RustString.from("Hello from Deno"));
    console.log(hello_string.into());
})
