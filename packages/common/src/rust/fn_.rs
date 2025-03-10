use crate::rust::util::*;
use crate::rust::{Attribute, Type};

/* -------------------------------------------------------------------------- */

// MARK: fn api

#[derive(Clone, Debug)]
pub struct ItemFn {
    pub attr:    Attribute,
    pub vis:     Visibility,
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
        let vis = input.parse()?;
        Self::parse_self_ty(input, attr, vis, None)
    }
}

impl ItemFn {
    pub fn parse_self_ty(
        input: ParseStream,
        attr: Attribute,
        vis: Visibility,
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
        Self::parse_remaining(input, attr, vis, self_ty, const_, unsafe_)
    }

    /// `parse_remaining` is used by the `item` parser since it has already
    /// parsed some attribute or visibility for both item_fn and item_impl
    pub fn parse_remaining(
        input: ParseStream,
        mut attr: Attribute,
        vis: Visibility,
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

        // [!TODO] rewrite this receiver parser

        if content.peek(Token![self])
        || content.peek(Token![mut]) && content.peek2(Token![self])
        || content.peek(Token![&]) && content.peek2(Token![self])
        || content.peek(Token![&]) && content.peek2(Token![mut]) && content.peek3(Token![self]) {
            let ty = Type::parse(&content, self_ty)?;
            assoc = Some(match &ty {
                Type::Ref(_) => Association::Instance,
                Type::RefMut(_) => Association::InstanceMut,
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

        let fork = input.fork();
        let output = if fork.parse::<Option<Token![->]>>()?.is_some() {
            input.advance_to(&fork);
            Type::parse(input, self_ty)?
        } else {
            Type::Void
        };

        if let Some(where_) = input.parse::<Option<Token![where]>>()? {
            return Err(Error::new(
                where_.span(),
                "generic parameters and where clauses are not supported",
            ));
        }

        // [!ISSUE] optimize parsing to skip checking of expressions and function
        // block's contents
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
            vis,
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
                | Type::Ptr(_)
                | Type::PtrMut(_)
                | Type::FnPtr(_) => (),
                Type::Ref(elem) => {
                    in_stmts.push(quote! { let #ident = unsafe { &*#ident }; });
                    *input = Type::Ptr(std::mem::take(elem));
                },
                Type::RefMut(elem) => {
                    in_stmts.push(quote! { let #ident = unsafe { &mut *#ident }; });
                    *input = Type::PtrMut(std::mem::take(elem));
                },
                Type::Box(elem) => {
                    in_stmts.push(
                        quote! { let #ident = unsafe { std::boxed::Box::from_raw(#ident) }; },
                    );
                    *input = Type::PtrMut(std::mem::take(elem));
                },
                rest => {
                    in_stmts.push(
                        quote! { let #ident = unsafe { *std::boxed::Box::from_raw(#ident) }; },
                    );
                    *rest = Type::PtrMut(Box::new(std::mem::take(rest)));
                },
            }
            args.push(ident);
        }

        *out_stmt = match output {
            Type::Void
            | Type::Numeric(_)
            | Type::Bool
            | Type::Char
            | Type::Ptr(_)
            | Type::PtrMut(_)
            | Type::FnPtr(_) => None,
            Type::Ref(elem) => {
                *output = Type::Ptr(std::mem::take(elem));
                Some(quote! { &raw const *out })
            },
            Type::RefMut(elem) => {
                *output = Type::PtrMut(std::mem::take(elem));
                Some(quote! { &raw mut *out })
            },
            Type::Box(elem) => {
                *output = Type::Ptr(std::mem::take(elem));
                Some(quote! { std::boxed::Box::into_raw(out) })
            },
            rest => {
                *rest = Type::Ptr(Box::new(std::mem::take(rest)));
                Some(quote! { std::boxed::Box::into_raw(std::boxed::Box::from(out)) })
            },
        };
    }
}

impl ToTokens for ItemFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ItemFn {
            vis,
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
            rest => {
                if out_stmt.is_some() {
                    call_expr = quote! { let out = #call_expr ( #(#call_args),* ); };
                } else {
                    call_expr = quote! { #call_expr ( #(#call_args),* ) };
                }
                quote! { -> #rest }
            },
        };

        let ident = match self_ty {
            Some(self_ty) => format_ident!("__{}__{}", self_ty.to_string(), ident.to_string()),
            None => format_ident!("__{}", ident.to_string()),
        };

        tokens.extend(quote! {
            #[unsafe(no_mangle)]
            #vis #const_ #unsafe_ extern "C" fn #ident ( #(#fn_args),* ) #output {
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

    macro_rules! pretty_test {
        ( { $( $source:tt )* }, { $( $expected:tt )* } ) => {
            println!("[source]\n\n{}", crate::prettify!(stringify!( $( $source )* )));

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
        println!(
            "{}",
            crate::prettify!(raw.to_token_stream().to_string().as_str())
        );

        let mut test_fn: ItemFn = syn::parse2(raw).unwrap();
        test_fn.transform();
        println!(
            "{}",
            crate::prettify!(test_fn.to_token_stream().to_string().as_str())
        );
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
