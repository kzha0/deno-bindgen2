use std::{io::Write, path::PathBuf};

use anyhow::format_err;
use dprint_plugin_typescript::{configuration::ConfigurationBuilder, format_parsed_source};

use crate::FFIData::*;

/*------------------------------------------------------------------------*/

type Result<T> = anyhow::Result<T>;

#[derive(Clone, Debug, Default)]
pub struct DenoParser {
    pub out_file:  Option<PathBuf>,
    pub cdylib:    String,
    pub lazy_init: Option<bool>,
    pub embed:   Option<bool>,
}

impl DenoParser {
    pub fn parse(&self, symbols: &'static [FFISymbol]) -> Result<()> {
        let mut ffi_feature_list = FFIModuleList::default();
        ffi_feature_list.parse_list(symbols);

        // cli prompt to check each detected module

        dbg!(ffi_feature_list);
        todo!();

        // call the code gen here
        let module = String::from("This module has not yet been built");

        let module = Ts::parse(module)?;
        let module = format_parsed_source(
            &module,
            &ConfigurationBuilder::new()
                .indent_width(4)
                .line_width(80)
                .build(),
        )
        .or_else(|err| Err(format_err!("Error formatting TypeScript output: {:?}", err)))?
        .unwrap_or(String::new());

        if let Some(out_file) = &self.out_file {
            write!(std::fs::File::create(out_file)?, "{module}")?;
        } else {
            write!(std::io::stdout(), "{module}")?;
        }

        Ok(())
    }
}

pub struct Ts {}
impl Ts {
    pub fn parse(source: String) -> Result<deno_ast::ParsedSource> {
        deno_ast::parse_module(deno_ast::ParseParams {
            specifier:      deno_ast::ModuleSpecifier::parse("")?,
            media_type:     deno_ast::MediaType::TypeScript,
            text:           deno_ast::SourceTextInfo::from_string(source).text(),
            capture_tokens: true,
            scope_analysis: false,
            maybe_syntax:   None,
        })
        .or_else(|err| Err(format_err!("Error parsing TypeScript code: {:?}", err)))
    }
}

// list that tracks which ffi types appear in user code
// used to select which interfacing modules to embed in the final module

#[derive(Clone, Debug, Default)]
struct FFIModuleList {
    char:         bool,
    reference:    bool,
    function_ptr: bool,
    str:          bool,
    string:       bool,
    slice:        bool,
    vec:          bool,
    _box:         bool,
    tuple:        bool,
}

#[rustfmt::skip]
impl FFIModuleList {
    fn parse_list(&mut self, symbols: &'static [FFISymbol]) {
        for symbol in symbols { match symbol {
            FFISymbol::Fn(ffi_function) => self.parse_function(ffi_function),
            FFISymbol::Impl(ffi_implement) => {
                for (_, ffi_function) in ffi_implement.methods {
                    self.parse_function(ffi_function);
                    if self.has_all() { return () }
                }
            },
        }}
    }
    fn parse_function(&mut self, ffi_function: &FFIFunction) {
        for input in ffi_function.sig.inputs {
            self.parse(input);
        }
        self.parse(&ffi_function.sig.output);
    }
    fn parse(&mut self, ffi_type: &FFIType) {
        match ffi_type {
            FFIType::Unsupported => (),
            FFIType::Primitive(_) => (),
            FFIType::Void => (),
            FFIType::Pointer(ffi_type) => {
                self.parse(ffi_type);
                self.reference = true;
            },
            FFIType::PointerMut(ffi_type) => {
                self.parse(ffi_type);
                self.reference = true;
            },
            FFIType::FunctionPointer(_) => {
                self.parse(ffi_type);
                self.reference = true;
            },
            FFIType::String => self.string = true,
            FFIType::Slice(ffi_type) => {
                self.parse(ffi_type);
                self.slice = true;
            },
            FFIType::Str => todo!(),
            FFIType::Vec(ffi_type) => {
                self.parse(ffi_type);
                self.vec = true;
            },
            FFIType::Box(ffi_type) => {
                self.parse(ffi_type);
                self._box = true;
            },
            FFIType::Tuple(ffi_types) => {
                for ffi_type in *ffi_types {
                    self.parse(ffi_type);
                }
                self.slice = true;
            },
            FFIType::UserDefined(_) => (),
        }
    }
    fn has_all(&self) -> bool {
        if self.char
        && self.reference
        && self.function_ptr
        && self.str
        && self.string
        && self.slice
        && self.vec
        && self._box
        && self.tuple {
            true
        } else {false }
    }
}
