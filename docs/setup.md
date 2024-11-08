# Setup

This project links directly to the rust compiler's (`rustc`) libraries as an optimization. Specifically, it uses the following crates:

- `rustc_interface` to invoke compiler-specific funtionality, such as type-checking
- `rustc_expand` to expand macro invocations [<sup>6</sup>](https://rustc-dev-guide.rust-lang.org/macro-expansion.html)
- `rustc_ast` to analyze code items

However, since those crates are private and unpublished on crates.io, the following steps must be taken to expose and use them:

- switch to the `nightly` toolchain using `rustup`
- add the `rustc-dev`[<sup>2</sup>](https://rust-lang.github.io/rustup/concepts/components.html) component

This lets you use the `rustc_*` crates for consuming modules. If you are using `rust-analyzer`, some additional steps[<sup>2</sup>](https://users.rust-lang.org/t/rust-analyzer-fails-to-index-due-to-unresolved-external-crate-in-a-rustc-private-project/105909/2?u=kzha0) must be taken to supress lint errors

- add the `"rust-analyzer.rustc.source": "discover"` to the from rust-analyzer config

- add the following property to the specific package using the extern crate

```toml
[package.metadata.rust-analyzer]
rustc_private = true`
```

This should now let you use rustc's crates without error lints from rust-analyzer

```rust
#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_interface;

...
```

Also, an additional component `llvm-tools-preview` must be added when using rust nightly, especially if the latest versions of tooling packages are not yet supported on the system:

```toml
[toolchain]
channel = "nightly"
components = ["rustc-dev", "llvm-tools-preview"]
```



Readings and references

<sup>1</sup> [`rustup` components](https://rust-lang.github.io/rustup/concepts/components.html)
<sup>2</sup> [`rust-analyzer` fails to index due to unresolved external crate in a `rustc` private project](https://users.rust-lang.org/t/rust-analyzer-fails-to-index-due-to-unresolved-external-crate-in-a-rustc-private-project/105909/2?u=kzha0)
<sup>3</sup> [Overview of the Compiler](https://rustc-dev-guide.rust-lang.org/overview.html)
<sup>4</sup> [The compiler source code](https://rustc-dev-guide.rust-lang.org/compiler-src.html)
<sup>5</sup> [Lexing and Parsing](https://rustc-dev-guide.rust-lang.org/the-parser.html)
<sup>6</sup> [Macro expansion](https://rustc-dev-guide.rust-lang.org/macro-expansion.html)
<sup>7</sup> [Example: Type checking through `rustc_interface`](https://rustc-dev-guide.rust-lang.org/rustc-driver/interacting-with-the-ast.html)
