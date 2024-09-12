# Naming convention

## Dash vs. underscore

The dash is used as a word separator when the project name is used in non-programming contexts.
On the other hand, the underscore separator is used in source code.

`deno-bindgen2` is a package name. It is different from the crate name `deno_bindgen2`. The compiler identifies crates, not packages. Either way, the compile will read dashes as underscores.