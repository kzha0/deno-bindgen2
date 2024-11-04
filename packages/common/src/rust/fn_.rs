use crate::rust::util::*;
use crate::rust::{Attribute, Type, TypeReference};

/* -------------------------------------------------------------------------- */

// MARK: fn api

#[derive(Clone, Debug)]
pub struct ItemFn {
    pub attr:    Attribute,
    pub const_:  Option<Token![const]>,
    pub unsafe_: Option<Token![unsafe]>,
    pub ident:   Ident,
    pub inputs:  Vec<Type>,
    pub output:  Type,
    pub assoc:   Option<Association>,
    pub block:   Block,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Association {
    Static,      // T -> T
    Constructor, // T -> Self with #[constructor]
    Instance,    // &Self -> T
    InstanceMut, // &mut Self -> T
    Destructor,  // Self -> T   requires `unsafe` qualifier
}

#[derive(Clone, Debug, Default)]
pub struct Block {
    pub args:     Vec<Ident>,
    pub in_stmts: Vec<TokenStream>,
    pub out_stmt: Option<TokenStream>,
    pub self_ty:  Option<Ident>,
}

// MARK: parse

impl Parse for ItemFn {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = Attribute::default();
        attr.parse_outer(input)?;
        input.parse::<Visibility>()?;
        Self::parse_self_ty(input, attr, None)
    }
}

impl ItemFn {
    pub fn parse_self_ty(
        input: ParseStream,
        attr: Attribute,
        self_ty: Option<&Ident>,
    ) -> Result<Self> {
        let const_ = input.parse()?;
        if let Some(async_) = input.parse::<Option<Token![async]>>()? {
            return Err(Error::new(
                async_.span,
                "unsupported async function qualifier",
            ));
        }
        let unsafe_ = input.parse()?;

        // TODO: should extern functions be restricted?

        if input.parse::<Option<Token![extern]>>()?.is_some() {
            if let Some(abi) = input.parse::<Option<LitStr>>()? {
                let abi_str = abi.value();
                if abi_str != "C" {
                    return Err(Error::new(
                        abi.span(),
                        "unsupported \"{abi_str}\" ABI type. replace with `extern \"C\"`, `extern`, or remove the `extern` keyword"
                    ));
                }
            }
        }
        input.parse::<Token![fn]>()?;
        Self::parse_remaining(input, attr, self_ty, const_, unsafe_)
    }

    /// `parse_remaining` is used by the `item` parser since it has already
    /// parsed some attribute or visibility for both item_fn and item_impl
    pub fn parse_remaining(
        input: ParseStream,
        mut attr: Attribute,
        self_ty: Option<&Ident>,
        const_: Option<Token![const]>,
        unsafe_: Option<Token![unsafe]>,
    ) -> Result<Self> {
        // continued after parsing the `fn` token
        let ident = input.parse::<Ident>()?;

        if let Some(lt_token) = input.parse::<Option<Token![<]>>()? {
            return Err(Error::new(
                lt_token.span(),
                "generic parameters are not supported",
            ));
        }

        let content;
        parenthesized!(content in input);
        let mut inputs = Vec::new();

        let mut assoc = if self_ty.is_some() {
            Some(Association::Static)
        } else {
            None
        };

        let has_self_value = content.peek(Token![self]);
        if has_self_value || content.peek(Token![&]) {
            if has_self_value && unsafe_.is_none() {
                // [!ISSUE] is this necessary? or is this verbosity appropriate
                // since the user is dealing with ffi boundaries
                let self_ = content.fork().parse::<Token![self]>()?;
                return Err(Error::new(
                    self_.span,
                    format!(
                        "this method takes ownership of `self`, which is unsafe. mark this function with unsafe: `unsafe fn {}`",
                        ident.to_string()
                    )
                ));
            }
            let ty = Type::parse(&content, self_ty)?;
            assoc = Some(match &ty {
                Type::Reference(TypeReference { mut_, .. }) => {
                    if *mut_ {
                        Association::InstanceMut
                    } else {
                        Association::Instance
                    }
                },
                Type::UserDefined(_) => Association::Destructor,
                _ => unreachable!(
                    "unknown error: parsing of self receiver did not return expected type"
                ),
            });
            inputs.push(ty);
            // parse trailing comma
            content.parse::<Option<Token![,]>>()?;
        }

        while !content.is_empty() {
            // discards any attribute if any
            content.call(syn::Attribute::parse_outer)?;
            Pat::parse_single(&content)?;
            content.parse::<Token![:]>()?;
            inputs.push(Type::parse(&content, self_ty)?);
            if content.is_empty() {
                break;
            }
            // allows for trailing comma as loop breaks if there is no token
            // after the comma (hence the `while !content.is_empty()` condition)
            content.parse::<Token![,]>()?;
        }

        let mut fork = input.fork();
        let output = if input.parse::<Option<Token![->]>>()?.is_some() {
            fork = input.fork();
            Type::parse(input, self_ty)?
        } else {
            Type::Void
        };

        if let Some(self_ty) = self_ty {
            if attr.has_constructor() {
                if !output.is_self_ty(self_ty) {
                    return Err(fork.error(format!(
                        "this function is marked `constructor` and should return `Self` or `{}`",
                        self_ty.to_string()
                    )));
                }

                if let Some(assoc_) = &assoc {
                    match assoc_ {
                        Association::Static => {
                            assoc = Some(Association::Constructor);
                        },
                        _ => (),
                    }
                }
            }
        }

        if let Some(where_) = input.parse::<Option<Token![where]>>()? {
            return Err(Error::new(
                where_.span(),
                "generic parameters and where clauses are not supported",
            ));
        }

        // [!ISSUE] is there a solution to skipping parsing the rest of the contents
        // of a function?
        // maybe parse until any bracket character is the last,
        // meaning it is either followed by an syn::Item, or is the end of the tt?????
        // but what if the item is enclosed as part of a larger delimiter
        let content;
        braced!(content in input);
        attr.parse_inner(&content)?;
        content.call(syn::Block::parse_within)?;

        let mut block = Block::default();
        if let Some(self_ty) = self_ty {
            block.self_ty = Some(self_ty.clone());
        }

        Ok(Self {
            attr,
            const_,
            unsafe_,
            ident,
            inputs,
            output,
            assoc,
            block,
        })
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn test_attrs_and_vis() {
        dbg_quote!(
            ItemFn,
            #[some_attr]
            pub fn test_fn() {}
        );
    }

    #[test]
    fn test_doc_attr_and_marker() {
        dbg_quote!(
            ItemFn,
            #[doc = "some documentation"]
            #[doc = "deno_bindgen"]
            #[constructor]
            pub fn test_fn() {}
        );
    }

    #[test]
    #[should_panic]
    fn test_async() {
        dbg_quote!(ItemFn, async fn test_fn() {});
    }

    #[test]
    fn test_const_unsafe() {
        dbg_quote!(ItemFn, const unsafe fn test_fn() {});
    }

    #[test]
    #[should_panic]
    fn test_generics() {
        dbg_quote!(ItemFn, fn test_fn<T>() {});
    }

    #[test]
    #[should_panic]
    fn test_where_clause() {
        dbg_quote!(
            ItemFn,
            fn test_fn()
            where
                T: Drop,
            {
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_self_lifetime() {
        dbg_quote!(ItemFn, fn test_fn(&'a self) {});
    }

    #[test]
    #[should_panic]
    fn test_self() {
        dbg_quote!(ItemFn, fn test_fn(&mut self) {});
    }

    #[test]
    #[should_panic]
    fn test_second_self() {
        dbg_quote!(ItemFn, fn test_fn(arg0: u8, arg1: Self) {});
    }

    #[test]
    fn test_args() {
        dbg_quote!(ItemFn, fn test_fn(arg0: usize) {});
    }

    #[test]
    fn test_many_args() {
        dbg_quote!(
            ItemFn,
            fn test_fn(
                arg0: usize,
                arg1: String,
                arg2: Box<usize>,
                arg3: Vec<String>,
                arg4: Vec<[str]>,
            ) -> () {
            }
        );
    }

    #[test]
    fn test_pattern_tup() {
        dbg_quote!(ItemFn, fn test_fn((x, y): (usize, u8)) {});
    }

    #[test]
    fn test_pattern_struct() {
        // note the type here will appear as unsupported as it is outside an impl block
        dbg_quote!(
            ItemFn,
            fn test_fn(SomeStruct { field_x, field_y }: SomeStruct) {}
        );
    }

    #[test]
    fn test_trailing_coma() {
        dbg_quote!(ItemFn, fn test_fn(arg0: usize, arg1: u8) {});
    }

    #[test]
    fn test_unit_return() {
        dbg_assert!(
            Type::Void,
            parse_quote!(ItemFn, fn test_fn() -> () {}).output
        );
    }

    #[test]
    fn test_return() {
        dbg_quote!(ItemFn, fn test_fn() -> Box<u8> {});
    }

    #[test]
    fn test_self_return() {
        // no need to do checking for the self type since the rust compiler
        // will handle this
        dbg_quote!(ItemFn, fn test_fn() -> Box<Self> {});
    }

    #[test]
    fn test_stmts() {
        dbg_quote!(
            ItemFn,
            fn test_fn() {
                let x = 0;
            }
        );
    }

    #[test]
    fn test_innter_attr() {
        dbg_quote!(
            ItemFn,
            fn test_fn() {
                #[some_attr] // doesn't need to be parsed?!
                let x = 0;
            }
        );
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl ItemFn {
    pub fn transform(&mut self) {
        let ItemFn {
            inputs,
            output,
            block,
            ..
        } = self;
        let Block {
            args,
            in_stmts,
            out_stmt,
            ..
        } = block;

        for (i, input) in inputs.iter_mut().enumerate() {
            let ident = format_ident!("arg_{i}");
            match input {
                Type::Void
                | Type::Numeric(_)
                | Type::Bool
                | Type::Char
                | Type::Pointer(_)
                | Type::FnPointer(_) => (),
                Type::Reference(TypeReference { mut_, elem }) => {
                    match *mut_ {
                        true => in_stmts.push(quote! { let #ident = unsafe { &mut *#ident }; }),
                        false => in_stmts.push(quote! { let #ident = unsafe { &*#ident }; }),
                    };
                    *input = Type::Pointer(TypeReference {
                        mut_: *mut_,
                        elem: std::mem::take(elem),
                    });
                },
                Type::Box(elem) => {
                    in_stmts.push(quote! { let #ident = unsafe { Box::from_raw(#ident) }; });
                    *input = Type::Pointer(TypeReference {
                        mut_: true,
                        elem: std::mem::take(elem),
                    });
                },
                _ => {
                    in_stmts.push(quote! { let #ident = unsafe { *Box::from_raw(#ident) }; });
                    *input = Type::Pointer(TypeReference {
                        mut_: true,
                        elem: Box::new(std::mem::take(input)),
                    });
                },
            }
            args.push(ident);
        }

        *out_stmt = match output {
            Type::Void
            | Type::Numeric(_)
            | Type::Bool
            | Type::Char
            | Type::Pointer(_)
            | Type::FnPointer(_) => None,
            Type::Reference(TypeReference { mut_, elem }) => {
                let mut_ = *mut_;
                *output = Type::Pointer(TypeReference {
                    mut_,
                    elem: std::mem::take(elem),
                });
                Some(match mut_ {
                    true => quote! { &raw mut *out },
                    false => quote! { &raw const *out  },
                })
            },
            Type::Box(elem) => {
                *output = Type::Pointer(TypeReference {
                    mut_: true,
                    elem: std::mem::take(elem),
                });
                Some(quote! { Box::into_raw(out) })
            },
            _ => {
                *output = Type::Pointer(TypeReference {
                    mut_: true,
                    elem: Box::new(std::mem::take(output)),
                });
                Some(quote! { Box::into_raw(Box::from(out)) })
            },
        };
    }
}

impl ToTokens for ItemFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ItemFn {
            const_,
            unsafe_,
            ident,
            inputs,
            output,
            block,
            ..
        } = self;
        let Block {
            args,
            in_stmts,
            out_stmt,
            self_ty,
        } = block;

        // omit unit `()` type from shim wrapper's parameters
        let mut fn_args = Vec::new();
        let mut call_args = Vec::new();
        for (i, input) in inputs.iter().enumerate() {
            if *input == Type::Void {
                call_args.push(quote! { () });
            } else {
                let arg = args.get(i).expect("error: this function's arguments have not been processed. call `transform` on this item");
                call_args.push(quote! { #arg });
                fn_args.push(quote! { #arg: #input });
            }
        }

        let mut call_expr = if let Some(self_ty) = self_ty {
            quote! { #self_ty :: #ident }
        } else {
            quote! { #ident }
        };
        let output = match output {
            Type::Void => {
                if out_stmt.is_some() {
                    panic!("this function has an out stmt but its signature must return void");
                }
                call_expr = quote! { #call_expr ( #(#call_args),* ); };
                TokenStream::new()
            },
            _ => {
                if out_stmt.is_some() {
                    call_expr = quote! { let out = #call_expr ( #(#call_args),* ); };
                } else {
                    call_expr = quote! { #call_expr ( #(#call_args),* ) };
                }
                quote! { -> #output }
            },
        };

        let ident = match self_ty {
            Some(self_ty) => format_ident!("__{}__{}", self_ty.to_string(), ident.to_string()),
            None => format_ident!("__{}", ident.to_string()),
        };

        tokens.extend(quote! {
            #[unsafe(no_mangle)]
            #const_ #unsafe_ extern "C" fn #ident ( #(#fn_args),* ) #output {
                #(#in_stmts)*
                #call_expr
                #out_stmt
            }
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

// macro expansion cannot be sensibly tested through rust's unit tests anymore
// need to move testing to an e2e context
#[cfg(test)]
mod print_tests {
    use super::*;

    #[macro_export]
    macro_rules! prettify {
        ( $input:expr ) => {{
            let out = syn::parse_file($input).unwrap();
            prettyplease::unparse(&out)
        }};
    }


    macro_rules! pretty_test {
        ( { $( $source:tt )* }, { $( $expected:tt )* } ) => {
            println!("[source]\n\n{}", prettify!(stringify!( $( $source )* )));

            let mut expanded = syn::parse2::<ItemFn>(quote::quote!{ $( $source )* })
                .map_err(|err| panic!("{err:#?}"))
                .unwrap();
            ItemFn::transform(&mut expanded);
            let expanded = crate::prettify!(expanded.into_token_stream().to_string().as_str());
            println!("[expanded]\n\n{expanded}");

            let expected = crate::prettify!(stringify!( $( $expected )* ));
            println!("[expected]\n\n{expected}");

            assert_eq!(expanded, expected);
        };
    }

    #[test]
    fn test_transform() {
        let raw = quote! {
            fn test_fn(arg0: String, arg1: Vec<Box<CustomType>>, arg2: () ) -> &str {}
        };
        println!("{}", raw.to_token_stream().to_string());

        let mut test_fn: ItemFn = syn::parse2(raw).unwrap();
        test_fn.transform();
        println!("{}", test_fn.to_token_stream().to_string());
    }

    #[test]
    fn test_pretty() {
        pretty_test!(
            {
                fn test_fn() {}
            },
            {
                #[unsafe(no_mangle)]
                extern "C" fn __test_fn() {
                    test_fn();
                }
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_non_fn() {
        pretty_test!(
            {
                const VAR: &'static str = "";
            },
            {}
        );
    }
}
