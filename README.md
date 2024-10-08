# deno-bindgen2

`deno-bindgen2` is an FFI generator that allows you to write idiomatic Rust code and consume it as an FFI library in Deno. It simplifies and automates FFI bindings generation for Rust to Deno so you don't have to think about it.

This project aims to empower TypeScript development by bridging access to Rust's powerful memory-safe ecosystem.

This is a fork of the original `deno_bindgen` project.

Currently a work-in-progress.

## To use

Add the `deno-bindgen2` crate to your library crate's dependencies. Take note of the dash `-` separator.

```toml
# Cargo.toml
[dependencies]
deno-bindgen2 = "0.1.0"
```

Import the `deno_bindgen2` attribute macro and apply it to a function item in your source code:

```rust
// lib.rs
use deno_bindgen2::deno_bindgen2;

#[deno_bindgen]
fn test_1() {
    println!("Hello, world!");
}

#[deno_bindgen]
fn test_2(str: &str) -> String {
    format!("{} to Rust!", str)
}
```

> Note:
>
> Currently, `deno-bindgen2` only supports a limited set of Rust types that can be idiomatically converted or passed between Rust/Deno contexts. Although eventual support for almost all types is planned.

Next, to generate the bindings and TypeScript code, you must install the CLI tool with the command below:
```bash
$ cargo install deno_bindgen2
```

Then, run this command in your project's package folder (not the workspace folder), preferrably with an argument `--out <path to output file>` to specify the output. Without an  `--out` argument, the TypeScript code will be printed to the console.

```bash
$ deno_bindgen2 --release --out ./lib/mod.ts
```

Finally, you can write your TypeScript code and import the functions with the same name/identity from the Rust code.

```ts
// hello_world.ts
import { test_1, test_2 } from "../test.ts";

test_1();

console.log(test_2("Hello from Deno"));
```

To run:

```bash
$ deno run --allow-all --unstable-ffi hello_world.ts
```

This should output:

```text
Hello, world!
Hello from Deno to Rust!
```
