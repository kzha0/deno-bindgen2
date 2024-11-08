# Roadmap


## Prototype

CLI

scan directory

- invoke cargo/rustc
- cli detects current package within nested subdirectories
- assert it is not at a workspace level
- parse cargo metadata of current package
- check if crate type is dylib
- get package name
- get package entrypoint file

- run unpretty=expanded on entrypoint file

- custom parser: read only items prefixed with attribute `#[deno_bindgen2]`, skip any after
- syn parse from file (optimization: how to buffer reads of token groups? rust lexer/parser that can parse from buf reading?)

- (debug) print list of items
- recursively visit function types to fill check list

- prompt user for confimration about feature list (autodetected features are turned on by default, the user can turn those off in this prompt)
- if no-interactive flag, auto-confirm features
- if no-parse flag, provide all features by default
- if no features, every non-trivial type is a fat pointer
- if no-remote flag, util modules will be embeded instead of being fetched remotely

- generaet cfg flags for dylib generation
- generate dylib file
- (debug) test symbols

- prepare for code generation
- check if -o flag was provided. if none, generate in default out diretory below
- create bin/out/build/target directory for generated dylib and ts module files



Code generation

- [ ] Support for documentation-in-code translation from rust to ts


