# Architecture

`deno-bindgen2` is a macro library for transforming Rust code into a Deno foreign function interface (FFI) library. This is a rewrite of the original `deno_bindgen` project.

At the highlest level, the macro basically works as a transpiler for converting some compatible Rust code into a version that works in Deno.

To achieve this, a transpilation process occurs which is divided into the following steps:

1. Parsing - tokens are covnerted to `deno-bindgen2`'s own syntax trees
2. Transformation - incompatible types are converted into a native/trivial type for FFI compatibility
3. Generation - a TypeScript module is automatically generated that includes the FFI library
