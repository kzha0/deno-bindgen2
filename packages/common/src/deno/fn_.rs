use crate::deno::util::*;
use crate::deno::{FfiFunction, RustType, TsModule};
use crate::rust::{Association, Attribute, ItemFn};

#[derive(Clone, Debug, Default)]
pub struct FunctionDefs {
    pub functions: Vec<TsMethod>,
}

#[derive(Clone, Debug)]
pub struct TsMethod {
    pub attr:      Attribute,
    pub ident:     Ident,
    pub inputs:    Vec<RustType>,
    pub output:    RustType,
    pub assoc:     Option<Association>,
    pub ffi_ident: Ident,
    pub self_ty:   Option<Ident>,
}

/* -------------------------------------------------------------------------- */

// MARK: transform

impl ItemFn {
    pub fn unwrap(self, module: &mut TsModule) -> TsMethod {
        // transform types

        let mut ffi_inputs = Vec::new();
        let mut inputs = Vec::new();
        for input in self.inputs {
            let (ffi_input, input) = input.unwrap(module);
            ffi_inputs.push(ffi_input);
            inputs.push(input);
        }

        let (ffi_output, output) = self.output.unwrap(module);

        let ident = self.ident;
        let ffi_ident = if let Some(self_ty) = &self.block.self_ty {
            format_ident!("__{}__{ident}", self_ty.to_string())
        } else {
            format_ident!("__{ident}")
        };

        // code generation
        module.ffi_lib.interface.push_fn(FfiFunction {
            ident:        ffi_ident.clone(),
            inputs:       ffi_inputs,
            output:       ffi_output,
            non_blocking: self.attr.has_non_blocking(),
        });

        TsMethod {
            attr: self.attr,
            ident,
            inputs,
            output,
            assoc: self.assoc.clone(),
            ffi_ident,
            self_ty: self.block.self_ty,
        }
    }
}

impl FunctionDefs {
    pub fn push(&mut self, method: TsMethod) {
        self.functions.push(method);
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl TsMethod {
    pub fn print(&self) -> TokenStream {
        let TsMethod {
            ref ident,
            ref inputs,
            ref output,
            ref assoc,
            ref ffi_ident,
            ref self_ty,
            ..
        } = self;

        let inputs: Vec<&RustType> = inputs
            .iter()
            .filter(|ty| if **ty == RustType::Void { false } else { true })
            .collect();


        let mut fn_args = Vec::new();
        let mut call_args = Vec::new();

        if !inputs.is_empty() {
            let inputs_slice = if let Some(assoc) = assoc {
                if self_ty.is_some() {
                    match assoc {
                        Association::Static => inputs.as_slice(),
                        _ => &inputs[1..inputs.len()],
                    }
                } else {
                    panic!("this method has an association but is missing a self type");
                }
            } else {
                inputs.as_slice()
            };

            for (i, input) in inputs_slice.iter().enumerate() {
                let fn_arg = format_ident!("arg_{i}");
                fn_args.push(quote! { #fn_arg: #input });

                match input {
                    RustType::Void => (),
                    rest => call_args.push(match rest {
                        RustType::Numeric(_)
                        | RustType::Boolean
                        | RustType::FnPtr(_)
                        | RustType::Ptr(_)
                        | RustType::PtrMut(_)
                        | RustType::Ref(_)
                        | RustType::RefMut(_)
                        | RustType::Unsupported => quote! { #fn_arg },
                        RustType::Char => quote! { #fn_arg.get() },
                        _ => quote! { #fn_arg.take() },
                    }),
                };
            }
        }

        let mut stmts = TokenStream::new();

        let mut ident = quote! { #ident };

        let fn_output;
        let call_expr = match output {
            RustType::Void => {
                fn_output = TokenStream::new();
                quote! { symbols.#ffi_ident }
            },
            rest => {
                if self.attr.has_non_blocking() {
                    ident = quote! { async #ident };
                    fn_output = quote! { : Promise<#rest> };
                    quote! { const out = await symbols.#ffi_ident }
                } else {
                    fn_output = quote! { : #rest };
                    quote! { const out = symbols.#ffi_ident }
                }
            },
        };

        let generate_return_stmt = || -> TokenStream {
            match &output {
                RustType::Void => TokenStream::new(),
                RustType::Numeric(_)
                | RustType::Boolean
                | RustType::FnPtr(_)
                | RustType::Ptr(_)
                | RustType::PtrMut(_)
                | RustType::Ref(_)
                | RustType::RefMut(_)
                | RustType::Unsupported => quote! {
                    return out! as #output;
                },
                rest => quote! {
                    return new #rest(out!) as #output;
                },
            }
        };

        if let Some(assoc) = assoc {
            stmts.extend(match assoc {
                Association::Static => {
                    ident = quote! { static #ident };
                    quote! {
                        #call_expr(#(#call_args),*);
                    }
                },

                Association::Instance => {
                    quote! {
                        #call_expr(this.ptr, #(#call_args),*);
                    }
                },
                Association::InstanceMut => {
                    quote! {
                        const ptr = this.ptr;
                        this.ptr = null;
                        #call_expr(ptr, #(#call_args),*)!;
                        this.ptr = ptr;
                    }
                },
                Association::Destructor => {
                    quote! {
                        const ptr = this.ptr;
                        this.ptr = null;
                        #call_expr(ptr, #(#call_args),*);
                    }
                },
            });
            stmts.extend(generate_return_stmt());
        } else {
            stmts.extend(quote! {
                #call_expr(#(#call_args),*);
            });
            stmts.extend(generate_return_stmt());
        }

        quote! {
            #ident(#(#fn_args),*) #fn_output {
                #stmts
            }

        }
    }
}

impl ToTokens for FunctionDefs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for method in &self.functions {
            let async_ = if method.attr.has_non_blocking() {
                quote! { async }
            } else {
                TokenStream::new()
            };

            let method = method.print();

            tokens.extend(quote! {
                export #async_ function #method
            });
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[cfg(test)]
mod print_tests {
    use super::*;
    use crate::deno::{TsFormat, TsModule};
    use crate::{parse_quote, prettify};

    macro_rules! test_transform {
        ($($tt:tt)*) => {
            println!("[rust source]\n{}", prettify!(stringify!($($tt)*)));

            let source = parse_quote!(ItemFn, $($tt)*);

            let mut export = source.clone();
            export.transform();
            println!("[rust wrapper]\n{}", prettify!(export.to_token_stream().to_string().as_str()));

            let mut module = TsModule::default();
            let method = source.unwrap(&mut module);
            module.functions.push(method);

            let ffi_lib = TsFormat::format(module.ffi_lib.to_token_stream().to_string());
            println!("[ts ffi]\n{}", ffi_lib);

            let functions = TsFormat::format(module.functions.to_token_stream().to_string());
            println!("[ts mod]\n{}", functions);
        };
    }

    #[test]
    fn test_empty() {
        test_transform!(
            fn test_fn() {}
        );
    }

    #[test]
    fn test_u8() {
        test_transform!(
            fn test_fn(arg0: u8) {}
        );
    }

    #[test]
    fn test_return() {
        test_transform!(
            fn test_fn(arg0: u8) -> u8 {}
        );
    }

    #[test]
    fn test_many_args() {
        test_transform!(
            fn test_fn(arg0: &mut [u8], arg1: &str, arg2: Box<CustomType>, arg3: Vec<String>) {}
        );
    }
}
