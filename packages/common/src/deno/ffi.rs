use crate::deno::util::*;

/* -------------------------------------------------------------------------- */

// MARK: api
// https://docs.deno.com/runtime/reference/deno_namespace_apis/#supported-types
#[derive(Clone, Debug, PartialEq)]
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

// https://docs.deno.com/api/deno/~/Deno.ForeignFunction
#[derive(Clone, Debug, PartialEq)]
pub struct FfiFunction {
    pub ident:        Ident,
    pub inputs:       Vec<FfiType>,
    pub output:       FfiType,
    pub non_blocking: bool, /* [!ISSUE] how to prevent data races and enforce integrity
                             * pub optional:     bool, */
}

// https://docs.deno.com/api/deno/~/Deno.ForeignLibraryInterface
#[derive(Clone, Debug, PartialEq)]
pub enum FfiSymbol {
    Function(FfiFunction),
    // Static(FfiStatic),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FfiInterface {
    pub symbols: Vec<FfiSymbol>,
}

impl FfiInterface {
    pub fn push_fn(&mut self, ffi_function: FfiFunction) {
        self.symbols.push(FfiSymbol::Function(ffi_function));
    }
    pub fn append(&mut self, other: &mut FfiInterface) {
        self.symbols.append(&mut other.symbols);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FfiLib {
    pub interface:  FfiInterface,
    pub dylib_path: String,
    pub lazy:       bool,
    pub export:     bool,
}

/* -------------------------------------------------------------------------- */

#[rustfmt::skip]
impl Parse for FfiType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str = input.parse::<LitStr>()?;
        let ffi_type = match lit_str.value().as_str() {
            "u8"       => FfiType::U8,
            "u16"      => FfiType::U16,
            "u32"      => FfiType::U32,
            "u64"      => FfiType::U64,
            "usize"    => FfiType::Usize,
            "i8"       => FfiType::I8,
            "i16"      => FfiType::I16,
            "i32"      => FfiType::I32,
            "i64"      => FfiType::I64,
            "isize"    => FfiType::Isize,
            "f32"      => FfiType::F32,
            "f64"      => FfiType::F64,
            "void"     => FfiType::Void,
            "bool"     => FfiType::Bool,
            "pointer"  => FfiType::Pointer,
            "buffer"   => FfiType::Buffer,
            "function" => FfiType::FnPointer,
            _ => return Err(syn::Error::new(lit_str.span(), "unknown ffi type"))
        };
        Ok(ffi_type)
    }
}

impl Parse for FfiFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // ident: {}
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // {...}
        let content;
        braced!(content in input);

        // parameters: []...
        let _ident = content.call(<Ident as syn::ext::IdentExt>::parse_any)?;
        if _ident.to_string().as_str() != "parameters" {
            return Err(syn::Error::new(_ident.span(), "expected key `parameters`"));
        }
        content.parse::<Token![:]>()?;

        // ["..."]
        let _content;
        bracketed!(_content in content);
        let mut inputs = Vec::new();
        while !_content.is_empty() {
            inputs.push(_content.parse()?);
            if _content.is_empty() {
                break;
            }
            _content.parse::<Token![,]>()?;
        }
        // [...],
        content.parse::<Token![,]>()?;

        let _ident = content.call(<Ident as syn::ext::IdentExt>::parse_any)?;
        if _ident.to_string().as_str() != "result" {
            return Err(syn::Error::new(_ident.span(), "expected key `result`"));
        }
        content.parse::<Token![:]>()?;
        let output = content.parse()?;
        content.parse::<Option<Token![,]>>()?;

        Ok(Self {
            ident,
            inputs,
            output,
            non_blocking: false,
        })
    }
}

impl Parse for FfiInterface {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut symbols = Vec::new();

        while !input.is_empty() {
            symbols.push(FfiSymbol::Function(input.parse::<FfiFunction>()?));
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self { symbols })
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

#[rustfmt::skip]
impl ToTokens for FfiType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            FfiType::U8        => quote! { "u8" },
            FfiType::U16       => quote! { "u16" },
            FfiType::U32       => quote! { "u32" },
            FfiType::U64       => quote! { "u64" },
            FfiType::Usize     => quote! { "usize" },
            FfiType::I8        => quote! { "i8" },
            FfiType::I16       => quote! { "i16" },
            FfiType::I32       => quote! { "i32" },
            FfiType::I64       => quote! { "i64" },
            FfiType::Isize     => quote! { "isize" },
            FfiType::F32       => quote! { "f32" },
            FfiType::F64       => quote! { "f64" },
            FfiType::Void      => quote! { "void" },
            FfiType::Bool      => quote! { "bool" },
            FfiType::Pointer   => quote! { "pointer" },
            FfiType::Buffer    => quote! { "buffer" },
            FfiType::FnPointer => quote! { "function" },
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

        let inputs: Vec<&FfiType> = inputs
            .iter()
            .filter(|ffi_type| {
                if **ffi_type == FfiType::Void {
                    false
                } else {
                    true
                }
            })
            .collect();

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

impl ToTokens for FfiSymbol {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            FfiSymbol::Function(ffi_function) => ffi_function.to_token_stream(),
        });
    }
}

impl ToTokens for FfiInterface {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let symbols = &self.symbols;
        tokens.extend(quote! {
            { #(#symbols),* }
        });
    }
}

impl<'a> ToTokens for FfiLib {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let FfiLib {
            interface,
            dylib_path,
            lazy,
            export,
        } = self;

        let export = if *export {
            quote! { export }
        } else {
            TokenStream::new()
        };

        tokens.extend(if *lazy {
            // [!TODO] refactor to a class?
            quote! {
                let symbols: any;
                export function load(path: string = #dylib_path) {
                    const { dlopen } = Deno;
                    const { symbols: symbols_ } = dlopen(path, #interface);
                    symbols = symbols_;
                };
            }
        } else {
            quote! {
                #export const { symbols } = Deno.dlopen(#dylib_path, #interface);
            }
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
