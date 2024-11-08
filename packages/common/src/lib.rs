#![cfg_attr(feature = "macro", feature(proc_macro_diagnostic))]
#![cfg_attr(feature = "cli", feature(btree_extract_if))]

#[cfg(feature = "macro")]
extern crate proc_macro;

#[allow(unused_imports)]
mod rust {
    mod attr;
    mod file;
    mod fn_;
    mod impl_;
    mod item;
    mod mod_;
    mod struct_;
    mod ty;
    mod util;
    pub use attr::{Attribute, Marker};
    pub use file::File;
    pub use fn_::{Association, ItemFn};
    pub use impl_::ItemImpl;
    pub use item::Item;
    pub use mod_::ItemMod;
    pub use struct_::ItemStruct;
    pub use ty::{Type, TypeNumeric};
}

#[cfg(feature = "cli")]
mod deno {
    mod class;
    mod ffi;
    mod file;
    mod fn_;
    mod ty;
    mod util;
    pub use class::ClassDefs;
    pub use ffi::{FfiFunction, FfiInterface, FfiLib, FfiType};
    pub use file::{CodegenOpts, TsModule};
    pub use fn_::{FunctionDefs, TsMethod};
    pub use ty::{RustType, RustTypeDefs, UserDefinedDefs};
    pub use util::TsFormat;
}

#[cfg(feature = "cli")]
pub use deno::{CodegenOpts, TsModule};
pub use rust::{File, Marker};
