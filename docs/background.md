# Background

One of the world's most popular and widely-used programming languages is JavaScript, owing to its ease of use and prevalence on the web.

The fact that JavaScript is a relatively easy to learn language is offsetted by the complexity surrounding JavaScript development, tooling, and the overwhelming amount of JavaScript "frameworks."

`Deno` is a modern runtime for JavaScript that addresses many of these quirks, along with new features that simplify and make JavaScript development a better experience.

To this end, the `deno-bindgen2` project enhances JavaScript development by providing access to system libraries through Rust. It provides a way to enhance interoperability between Rust and JavaScript and make native platform development a closer reality for JavaScript devs.

## Design goals

`deno-bindgen2` is an FFI bindings and generation tool for porting Rust libraries for use in Deno.

The main goal of this project is to minimize the complexity of writing an FFI library for Deno. It aims to abstract away all the things the user would normally worry about if they were to write an FFI library, such as:

- checking for FFI-safety and compatibility with the target language's FFI APIs
- translating one language construct to another, such as Rust's lack of "object composition", borrowing, and ownership

Through this, users can write code as close to idiomatic Rust using the same semantics and convention of the language and feel as if they were writing just another library for Rust, even though the intention is to use it outside Rust.

## Planned Features

This project plans to provide the following features

- [ ] Rust source code parser
- [ ] Comprehensive support for Rust types
- [ ] Typescript code generation CLI tool
- [ ] Code generation options and granular control
- [ ] Incremental builds and code generation
- [ ] Zero overhead bindings
- [ ] Insights about code vulnerabilities and possible safety violations of Rust's programming model
- [ ] Utility modules for interating with Rust data structures
