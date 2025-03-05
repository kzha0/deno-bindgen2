# deno-bindgen2

> [!WARNING]
> Work in Progress 🚧
>
> This documentation is currently incomplete and still a work in progress

`deno-bindgen2` is an FFI bindings generator that simplifies writing Rust libraries for Deno.

It works by providing procedural macros that create wrappers around native Rust functions and make it FFI-safe. This makes writing FFI libraries easier as it deals with boilerplate code generation so you don't have to.

It also provides a CLI tool that generates a TypeScript module and tries to follow the semantics of your code wherever possible.

This project works along with the `deno-bindgen2-utils` library (to be) published on JSR, which contains utilities for interfacing with Rust's data structures in TypeScript.

This project aims to empower TypeScript development by bridging access to Rust's powerful memory-safe ecosystem.

## To use

This library depends on nightly rust features. It is recommended to set your project on the nightly tool chain as well to make the tool work as intended.

Add the `deno-bindgen2` crate to your library crate's dependencies (note the spelling and dash `-` separator)

```toml
# Cargo.toml
[dependencies]
deno-bindgen2 = "1.0"
```

Import *everything* from `deno_bindgen2` and use the `deno_bindgen` attribute macro on a function, implement, or struct item in your source code:

```rust
// lib.rs
use deno_bindgen2::*;

#[deno_bindgen]
fn test_1() {
    println!("Hello, world!");
}

#[deno_bindgen]
fn test_2(string: String) -> String {
    format!("{} to Rust!", str)
}
```

> [!NOTE]
>
> Currently, `deno-bindgen2` only supports a limited set of Rust types that can be idiomatically converted or passed between Rust/Deno contexts. Although eventual support for all possible types is planned.
>
> See the [limitations](docs/limitations.md) documentation for more info

Next, to generate the bindings and TypeScript code, you must install the CLI tool with the command below:

```sh
cargo install deno-bindgen2-cli
```

Then, run this command in your project's package folder (not the workspace folder).

```sh
deno-bindgen2 --release
```

This will automatically generate a TypeScript module in `<pkg_root>/dist/<your_module>`, along with another module `rust_type.ts` that contains TypeScript representations of Rust types.

Finally, you can write your TypeScript code and import the functions with the same name/identity from the Rust code.

```ts
// hello_world.ts
import { test_1, test_2 } from "./dist/libmy_mod.ts";
import { RustString } from "./dist/rust_type.ts";

test_1();

Deno.test("test_string", () => {
    // create a `RustString` instance from a JavaScript string
    // by calling the `from()` static method
    const hello_string = test_2(RustString.from("Hello from Deno"));

    // turn the RustString into a JavaScript string
    // by calling its `into()` method
    console.log(hello_string.into());
})

```

To run:

```bash
deno run --allow-all --unstable-ffi hello_world.ts
```

This should output:

```text
Hello, world!
Hello from Deno to Rust!
```

For additional code generation options, run `deno-bindgen2 --help`

## Performance

This project improves proc macro performance by [as much as 15x](packages/bench/bench_data.md) over `deno_bindgen`, reducing processing time from ~0.30 ms to ~0.02 ms
