# Task list

## Todo

compilation & performance

- [ ] optimize compilation times
- [ ] cache builds
- [ ] find a way to circumvent double builds when running cargo expand
- [ ] reuse build artifacts
- [ ] buffer reads/writes
- [ ] skip parsing of inner block expressions
- [ ] terminate early if syntax parsing error encountered, with no compile errors (this will be handled by rust-analyzer or rustc directly). just emit the unmodified token stream

feature & type support

- [ ] support for documentation in code (doc attributes)
- [ ] support for generic collection types (slice, vec) - possible monomorphization solution
- [ ] support for tuples
- [ ] supoort for path types and scoped imports analysis
- [ ] possible configuration file (toml/jsonc?)
- [ ] create interactive interface
- [ ] custom error types (syntax parsing error without a compilation error, deno-bindgen2 error with an error message/warning) in line with above

maintainability

- [ ] write all constants in separate module (i.e. reserved words, error messages, pregenerated code)
- [ ] write comprehensive unit & integration tests
- [ ] improve documentation
- [ ] write examples
- [ ] write technical illustrations
- [ ] promote

## Done

- [x] optimized syn parser function and macro
- [x] expanded source code analysis
- [x] OOP-like construct support (implements and classes)
- [x] custom representation of standard rust types
