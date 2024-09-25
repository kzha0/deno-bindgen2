// This is the 'gateway' module that links the public API and the internal source code
// This also exposes the internal modules to each other

/*======= EXTERNAL CRATE IMPORTS =======*/

pub use proc_macro::TokenStream;
pub use syn::Item;

/*======= INTERNAL MODULES =======*/

pub mod ir;
pub mod parse;
// pub mod transform;
pub mod generate;
pub mod util;

pub use ir::*;
pub use parse::*;
// pub use transform::*;
pub use generate::*;
pub use util::*;
