


// Polymorpic constraints/patterns for parser/transform/generate code




// create list of error codes here

// Unsupported:
// `deno_bindgen2` currently has no way of handling {} code

// Note: in the future, once `proc_macro::Diagnostic` is stabilized and supports spans with code suggestions, refactor the error reporter to provide code suggestions

// create a domain sub language (DSL) that supports creating TypeScript code in Rust
// maybe use SWC to create an AST, and use a macro to parse TS code into an AST
