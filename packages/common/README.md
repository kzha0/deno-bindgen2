# deno-bindgen2-common

This library contains common code for parsing and transforming Rust source code. It has two primary consumers: `deno-bindgen2-macro` and `deno-bindgen2-cli`

The macro crate uses this library to parse and transform source code and generate FFI symbol wrappers. The cli crate uses this library to parse Rust source code and generate a close analogue for TypeScript.
