use std::path::PathBuf;

use crate::deno::util::*;
use crate::deno::{ClassDefs, FfiLib, FunctionDefs, RustTypeDefs, TsFormat, UserDefinedDefs};
use crate::rust::{File, Item, ItemMod};

/* -------------------------------------------------------------------------- */

#[derive(Clone, Debug)]
pub struct CodegenOpts {
    pub file_name:  String,
    pub dylib_path: String,
    pub lazy:       bool,
    /// If false, uses the opaque representations of rust types with no method
    /// to interface with rust data structures
    ///
    /// If true, uses the extended rust types with methods for interfacing
    /// with rust data structures, but embeds the ffi symbols on the same dylib
    pub extended:   bool,
    /// If provided, writes the extended rust types on a  separate
    /// file and uses the dylib from this path for the typescript representation
    /// of the extended rust types. Incompatible with `inline=true`
    pub embedded:   Option<PathBuf>,
}

#[derive(Clone, Debug, Default)]
pub struct TsModule {
    pub ffi_lib:    FfiLib, // symbol definitions
    pub type_defs:  RustTypeDefs, /* import statements or type definitions if `inline = true`.
                             * links to standard types */
    pub user_defs:  UserDefinedDefs,
    pub functions:  FunctionDefs,
    pub class_defs: ClassDefs,
}

// TODO: use buffer where possible to handle large projects

impl TsModule {
    pub fn new(file: File, opts: &CodegenOpts) -> Self {
        let mut module = TsModule::default();
        module.ffi_lib.dylib_path = opts.dylib_path.clone();
        module.ffi_lib.lazy = opts.lazy;

        module.unwrap(file.items);
        module.user_defs.dedup(&module.class_defs);
        module
    }

    /// Recursively transforms parsed rust items into their typescript
    /// representations
    fn unwrap(&mut self, items: Vec<Item>) {
        for item in items {
            match item {
                Item::Fn(item_fn) => {
                    let method = item_fn.unwrap(self);
                    self.functions.push(method);
                },
                Item::Impl(item_impl) => {
                    item_impl.unwrap(self);
                },
                Item::Mod(ItemMod { items, .. }) => {
                    self.unwrap(items);
                },
                _ => (),
            }
        }
    }

    /// Generate a single typescript module with rust type definitions inlined
    pub fn generate_single(self, opts: &CodegenOpts) -> String {
        let TsModule {
            mut ffi_lib,
            mut type_defs,
            user_defs,
            functions,
            class_defs,
        } = self;

        type_defs.extended = opts.extended;
        let (type_defs, mut ffi_symbols) = type_defs.print_inline();
        ffi_lib.interface.append(&mut ffi_symbols);

        let ffi_lib = ffi_lib.to_token_stream().to_string();
        let user_defs = user_defs.to_token_stream().to_string();
        let functions = functions.to_token_stream().to_string();
        let class_defs = class_defs.to_token_stream().to_string();

        TsFormat::format(format!(
            "// deno-lint-ignore-file\n
            {ffi_lib}
            {type_defs}
            {user_defs}
            {class_defs}
            {functions}
            "
        ))
    }

    /// Generate multiple typescript modules with rust type definitions on a
    /// separate file
    pub fn generate_multiple(
        mut self,
        opts: &CodegenOpts,
        type_defs_name: &str,
    ) -> (String, String) {
        let TsModule {
            mut ffi_lib,
            user_defs,
            functions,
            class_defs,
            ..
        } = self;


        self.type_defs.extended = opts.extended;

        let (raw_type_defs, mut ffi_symbols) = self.type_defs.print_inline();
        let mut type_defs_builder = String::from(format!("// deno-lint-ignore-file\n"));

        // use extended types that requires symbols
        if opts.extended {
            // use a separate dylib file with a dlopen statement
            if let Some(embedded) = &opts.embedded {
                let mut embedded_ffi_lib = FfiLib::default();

                embedded_ffi_lib.dylib_path = embedded
                    .to_str()
                    .expect("unknown utf8 character on embedded dylib path")
                    .to_string();
                embedded_ffi_lib.interface.append(&mut ffi_symbols);

                let embedded_ffi_lib =
                    TsFormat::format(embedded_ffi_lib.to_token_stream().to_string());

                // append `const { symbols } = Deno.dlopen(...`
                type_defs_builder.push_str(&embedded_ffi_lib);

            // use an import { symbols } statement
            } else {
                ffi_lib.interface.append(&mut ffi_symbols);
                ffi_lib.export = true;

                let file_name = format!("./{}", opts.file_name);

                type_defs_builder.push_str(&TsFormat::format(
                    quote! {
                        import { symbols } from #file_name;
                    }
                    .to_string(),
                ));
            }
        }

        type_defs_builder.push_str(&raw_type_defs);

        let (type_defs, imports) = self
            .type_defs
            .print_separate(type_defs_builder, type_defs_name);

        let ffi_lib = ffi_lib.to_token_stream().to_string();
        let user_defs = user_defs.to_token_stream().to_string();
        let functions = functions.to_token_stream().to_string();
        let class_defs = class_defs.to_token_stream().to_string();

        let module = TsFormat::format(format!(
            "
            {imports}
            {ffi_lib}
            {user_defs}
            {class_defs}
            {functions}
            "
        ));

        (module, type_defs)
    }
}
