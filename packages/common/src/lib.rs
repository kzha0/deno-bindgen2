#![cfg_attr(feature = "macro", feature(proc_macro_diagnostic))]
#[cfg(feature = "macro")]
extern crate proc_macro;

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
    pub use fn_::{Association, Block, ItemFn};
    pub use impl_::ItemImpl;
    pub use item::Item;
    pub use mod_::ItemMod;
    pub use struct_::ItemStruct;
    pub use ty::{Type, TypeNumeric, TypeReference};
}

// #[cfg(feature = "cli")]
mod deno {
    mod class;
    mod ffi;
    mod file;
    mod fn_;
    mod ty;
    mod util;
    pub use class::{Classes, TsClass};
    pub use ffi::{FfiFunction, FfiType};
    pub use fn_::{TsFunction, TsMethod};
    pub use ty::{RustType, RustTypeList, TsType, TypedArray, UserDefined};
}



pub use rust::{File, Marker};
