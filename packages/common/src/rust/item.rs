use crate::fn_::ItemFn;
use crate::impl_::ItemImpl;
use crate::util::*;

/* -------------------------------------------------------------------------- */

// MARK: item api

#[derive(Clone, Debug)]
pub enum Item {
    Fn(ItemFn),
    Impl(ItemImpl),
    // Struct(ItemStruct), // [!TODO] support structs to follow class fields visibility from rust
    // Static(ItemStatic), // [!TODO] support static items and data
}

// MARK: parse

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Visibility>()?;

        // https://doc.rust-lang.org/cargo/reference/features.html?highlight=featu#command-line-feature-options
        // control conditional compilation through granular feature flags for downstream
        // users (`cli` and `macro` crates)

        // start a speculative parse to check if there is an `fn` token
        let fork = input.fork();
        let const_ = fork.parse::<Option<Token![const]>>();
        let async_ = fork.parse::<Option<Token![async]>>();
        let extern_ = fork.parse::<Option<Token![extern]>>();
        let abi = fork.parse::<Option<LitStr>>();
        let unsafe_ = fork.parse::<Option<Token![unsafe]>>();
        let fn_ = fork.parse::<Token![fn]>();
        if const_.is_ok()
            && async_.is_ok()
            && extern_.is_ok()
            && abi.is_ok()
            && unsafe_.is_ok()
            && fn_.is_ok()
        {
            let async_ = async_.unwrap();
            if let Some(async_) = async_ {
                return Err(Error::new(
                    async_.span,
                    "unsupported async function",
                ));
            } else if extern_.unwrap().is_some() {

                // TODO: should extern functions be restricted?

                if let Some(abi) = abi.unwrap() {
                    let abi_str = abi.value();
                    if abi_str != "C" {
                        return Err(Error::new(
                            abi.span(),
                            "unsupported \"{abi_str}\" ABI type. replace with `extern \"C\"`, `extern`, or remove the `extern` keyword"
                        ));
                    }
                }
            }
            input.advance_to(&fork);
            let const_ = const_.unwrap();
            let unsafe_ = unsafe_.unwrap();
            return Ok(Self::Fn(ItemFn::parse_with_progress(input, attrs, None, const_, unsafe_)?));
        }

        // start a speculative parse to check if there is an `impl` token
        let fork = input.fork();
        let unsafe_ = fork.parse::<Option<Token![unsafe]>>();
        let impl_ = fork.parse::<Token![impl]>();
        if unsafe_.is_ok() && impl_.is_ok() {
            input.advance_to(&fork);
            let unsafe_ = unsafe_.unwrap();
            return Ok(Self::Impl(ItemImpl::parse_with_progress(input, attrs, unsafe_ )?));
        }

        Err(input.error("failed to parse item: expected `fn`, `impl`"))
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn test_item_fn() {
        dbg_quote!(Item,
            fn test_fn() {}
        );
    }

    #[test]
    fn test_item_fn_with_attrs_and_vis() {
        dbg_quote!(Item,
            #[outer_attr]
            pub fn test_fn() {
                #![innter_attr]
            }
        );
    }

    #[test]
    fn test_item_impl_() {
        dbg_quote!(Item,
            impl CustomType {
                fn test_fn() {}
            }
        );
    }

    #[test]
    fn test_item_impl_with_attrs_and_vis() {
        dbg_quote!(Item,
            #[outer_attr]
            pub impl CustomType {
                #![innter_attr]
                #[outer_attr]
                pub fn test_fn() {
                    #![innter_attr]
                }
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_other_item() {
        dbg_quote!(Item,
            const _: () = {};
        );
    }
}



// #[proc_macro_attribute]
// pub fn test_attr(attr: TokenStream, _: TokenStream) -> TokenStream {
//     let attrs = parse_macro_input!(attr as MacroAttribute);
//     panic!("{attrs:#?}");
// }

// #[derive(Debug)]
// struct MacroAttribute {
//     meta: Meta
// }

// impl Parse for MacroAttribute {
//     fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//         Ok(Self { meta: input.parse()? })
//     }
// }

/* -------------------------------------------------------------------------- */

// MARK: print

impl Item {
    pub fn parse_macro_input(attr: TokenStream, input: TokenStream) -> TokenStream {
        let source = input.clone();

        let mut item: Item = match syn::parse2(input) {
            Ok(item) => item,
            Err(err) => return err.to_compile_error(),
        };

        // TODO: move from doc attributes to inert attributes
        // support custom inert attributes rfc
        // https://github.com/rust-lang/rust/issues/66079

        item.to_ffi_safe();
        quote! {
            #[cfg_attr(not(deno_bindgen), doc = "deno_bindgen")]
            #source
            #item
        }
    }

    fn to_ffi_safe(&mut self) {
        match self {
            Item::Fn(item_fn) => item_fn.to_ffi_safe(),
            Item::Impl(item_impl) => item_impl.to_ffi_safe(),
        }
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Item::Fn(item_fn) => item_fn.to_token_stream(),
            Item::Impl(item_impl) => item_impl.to_token_stream(),
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[cfg(test)]
mod print_tests {

}
