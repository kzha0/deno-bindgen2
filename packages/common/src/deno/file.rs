use crate::deno::ffi::FfiSymbol;
use crate::deno::{Classes, RustTypeList, TsFunction, UserDefined};
use crate::rust::{File, Item, ItemMod};

/* -------------------------------------------------------------------------- */

#[derive(Clone, Debug)]
pub struct CodegenOpts {
    pub dylib_path:  String,
    pub lazy:        bool,
    pub embed_utils: bool,
}

#[derive(Clone, Debug)]
pub enum UtilOpts {
    None,   // creates opaque pointers
    Embed,  // inlines utility modules
    Remote, // fetches utility modules from jsr
}

// INSERT STRUCT TO TRACK LIST OF MODULE ITEMS

#[derive(Clone, Debug, Default)]
pub struct Module {
    pub symbols: Vec<FfiSymbol>,
    pub exports: Export,
}

#[derive(Clone, Debug, Default)]
pub struct Export {
    pub functions:    Vec<TsFunction>,
    pub classes:      Classes,
    pub user_defined: UserDefined,
    pub rust_types:   RustTypeList, /* pub module: Module // namespace? or export const
                                     * pub static: ?? */
}

// TODO: use buffer where possible to handle large projects

impl Module {
    pub fn unwrap(&mut self, items: Vec<Item>) {
        for item in items {
            let ref mut symbols = self.symbols;
            let Export {
                ref mut functions,
                ref mut classes,
                ref mut user_defined,
                ref mut rust_types,
            } = self.exports;

            match item {
                Item::Fn(item_fn) => {
                    let (ffi_function, ts_method) = item_fn.unwrap(rust_types, user_defined);
                    symbols.push(FfiSymbol::Function(ffi_function));
                    functions.push(TsFunction::from(ts_method));
                },
                Item::Impl(item_impl) => {
                    let mut ffi_functions = Vec::new();
                    let mut ts_methods = Vec::new();
                    for item in item_impl.items {
                        let (ffi_function, ts_method) = item.unwrap(rust_types, user_defined);
                        ffi_functions.push(ffi_function);
                        ts_methods.push(ts_method);
                    }

                    let mut ffi_symbols = Vec::new();
                    for ffi_function in ffi_functions {
                        ffi_symbols.push(FfiSymbol::Function(ffi_function));
                    }

                    classes.append(item_impl.self_ty.to_string().as_str(), ts_methods);
                    symbols.append(&mut ffi_symbols);
                },
                Item::Mod(ItemMod { items, .. }) => {
                    self.unwrap(items);
                },
                _ => (),
            }
        }
    }


    pub fn generate(file: File, opts: CodegenOpts) -> String {
        // assess codegen opts first
        // get list of utility modules

        // get list of user defined types
        // if itemfn contains unsupported(type) that matches one of user defined types,
        // replace it with that

        // generate list of symbols
        // generate exports

        let module = Module::default();

        todo!()
    }
}
