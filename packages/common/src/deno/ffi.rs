use std::path::PathBuf;

use crate::deno::util::*;

/* -------------------------------------------------------------------------- */

// MARK: api

#[derive(Clone, Debug, Default)]
pub struct FfiLib {
    pub symbols:    Vec<FfiSymbol>,
    pub dylib_path: Option<PathBuf>,
    pub lazy:       bool,
}

// https://docs.deno.com/api/deno/~/Deno.ForeignLibraryInterface
#[derive(Clone, Debug)]
pub enum FfiSymbol {
    Function(FfiFunction),
    // Static(FfiStatic),
}

// https://docs.deno.com/api/deno/~/Deno.ForeignFunction
#[derive(Clone, Debug)]
pub struct FfiFunction {
    pub ident:        Ident,
    pub inputs:       Vec<FfiType>,
    pub output:       FfiType,
    pub non_blocking: bool, /* [!ISSUE] how to prevent data races and enforce integrity
                             * pub optional:     bool, */
}

// https://docs.deno.com/runtime/reference/deno_namespace_apis/#supported-types
#[derive(Clone, Debug)]
pub enum FfiType {
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
    F32,
    F64,
    Void,
    Bool,
    Pointer,
    Buffer, /* cannot be constructed directly. must be passed through the utility modules for
             * conversion from opaque pointer to access internal buffer */
    FnPointer,
    // Struct, // [!TODO] support for struct type
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl ToTokens for FfiLib {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FfiLib {
            symbols,
            dylib_path,
            lazy,
        } = self;

        let dylib_path = if let Some(dylib_path) = dylib_path {
            dylib_path.display().to_string()
        } else {
            format!("")
        };

        tokens.extend(if *lazy {
            // [!TODO] refactor to a class?
            quote! {
                let symbols: any;
                export function load(path: string = #dylib_path {
                    const {{ dlopen }} = Deno;
                    const {{ symbols: symbols_ }} = dlopen(path, {
                        #(#symbols),*
                    });
                });
            }
        } else {
            quote! {
                const { symbols } = Deno.dlopen(#dylib_path, {
                    #(#symbols),*
                });
            }
        });
    }
}


impl ToTokens for FfiSymbol {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            FfiSymbol::Function(ffi_function) => ffi_function.to_token_stream(),
        });
    }
}

impl ToTokens for FfiFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FfiFunction {
            ident,
            inputs,
            output,
            non_blocking,
        } = self;

        let inputs = quote! { #(#inputs),* };
        let non_blocking = if *non_blocking {
            quote! { non_blocking: true }
        } else {
            TokenStream::new()
        };

        tokens.extend(quote! {
            #ident: {
                parameters: [#inputs],
                result: #output,
                #non_blocking
            }
        });
    }
}

#[rustfmt::ski[]]
impl ToTokens for FfiType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            FfiType::U8 => quote! { "u8" },
            FfiType::U16 => quote! { "u16" },
            FfiType::U32 => quote! { "u32" },
            FfiType::U64 => quote! { "u64" },
            FfiType::Usize => quote! { "usize" },
            FfiType::I8 => quote! { "i8" },
            FfiType::I16 => quote! { "i16" },
            FfiType::I32 => quote! { "i32" },
            FfiType::I64 => quote! { "i64" },
            FfiType::Isize => quote! { "isize" },
            FfiType::F32 => quote! { "f32" },
            FfiType::F64 => quote! { "f64" },
            FfiType::Void => quote! { "void" },
            FfiType::Bool => quote! { "bool" },
            FfiType::Pointer => quote! { "pointer" },
            FfiType::Buffer => quote! { "buffer" },
            FfiType::FnPointer => quote! { "function" },
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print() {
        let ffi_function = FfiFunction {
            ident:        format_ident!("some_symbol"),
            inputs:       vec![FfiType::Pointer, FfiType::U8],
            output:       FfiType::Void,
            non_blocking: false,
        };
        println!("{}", ffi_function.to_token_stream().to_string());
    }
}
