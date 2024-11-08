use crate::rust::util::*;
use crate::rust::{Attribute, ItemFn, ItemImpl, ItemMod, ItemStruct};

/* -------------------------------------------------------------------------- */

// MARK: item api

#[derive(Clone, Debug)]
pub enum Item {
    Fn(ItemFn),
    Impl(ItemImpl),
    Mod(ItemMod),
    Struct(ItemStruct),
    // Static(ItemStatic), // [!TODO] support static items and data
}

// MARK: parse

impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::parse_remaining(input, false)
    }
}

impl Item {
    fn parse_remaining(input: ParseStream, filtered: bool) -> Result<Self> {
        let mut attr = Attribute::default();
        attr.parse_outer(input)?;
        let vis = input.parse()?;

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
                return Err(Error::new(async_.span, "unsupported async function"));
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
            return Ok(Self::Fn(ItemFn::parse_remaining(
                input, attr, vis, None, const_, unsafe_,
            )?));
        }

        // start a speculative parse to check if there is an `impl` token
        let fork = input.fork();
        let unsafe_ = fork.parse::<Option<Token![unsafe]>>();
        let impl_ = fork.parse::<Token![impl]>();
        if unsafe_.is_ok() && impl_.is_ok() {
            input.advance_to(&fork);
            let unsafe_ = unsafe_.unwrap();
            return Ok(Self::Impl(ItemImpl::parse_remaining(input, attr, unsafe_)?));
        }

        let fork = input.fork();
        let struct_ = fork.parse::<Token![struct]>();
        if struct_.is_ok() {
            input.advance_to(&fork);
            return Ok(Self::Struct(ItemStruct::parse_remaining(input, attr)?));
        }

        if cfg!(not(feature = "macro")) {
            let fork = input.fork();
            let unsafe_ = fork.parse::<Option<Token![unsafe]>>();
            let mod_ = fork.parse::<Token![mod]>();
            if unsafe_.is_ok() && mod_.is_ok() {
                input.advance_to(&fork);
                return Ok(Self::Mod(ItemMod::parse_remaining(input, attr, filtered)?));
            }
        }

        Err(input.error("failed to parse item: expected `fn`, `impl`"))
    }

    pub fn parse_many(input: ParseStream, filtered: bool) -> Result<Vec<Self>> {
        let mut items = Vec::new();

        while !input.is_empty() {
            // `filtered` is used by the mod parser which comes back to this function
            let fork = input.fork();
            if let Ok(item) = Item::parse_remaining(&fork, filtered) {
                input.advance_to(&fork);
                if filtered {
                    // filter only `deno_bindgen` items
                    match &item {
                        Item::Fn(ItemFn { attr, .. })
                        | Item::Impl(ItemImpl { attr, .. })
                        | Item::Struct(ItemStruct { attr, .. }) => {
                            if attr.has_deno_bindgen() {
                                items.push(item)
                            }
                        },
                        Item::Mod(_) => items.push(item),
                    }
                } else {
                    items.push(item);
                }
            } else {
                input.call(syn::Item::parse)?;
            }
        }

        Ok(items)
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn test_item_fn() {
        dbg_quote!(Item, fn test_fn() {});
    }

    #[test]
    fn test_item_fn_with_attrs_and_vis() {
        dbg_quote!(
            Item,
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
        dbg_quote!(
            Item,
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
    fn test_struct() {
        dbg_quote!(Item, struct CustomType;);
    }

    #[test]
    #[should_panic]
    #[cfg(feature = "macro")]
    fn test_unsupported() {
        dbg_quote!(Item, const _: () = {};);
    }

    #[test]
    fn test_full() {
        dbg_quote!(
            Item,
            #[this_mod]
            mod my_mod {
                #[doc = "deno_bindgen"]
                #[doc = "some_documentation"]
                fn my_fn() {}

                fn ignored_function() {}

                #[doc = "deno_bindgen"]
                struct CustomType {}

                struct IgnoredStruct {}

                #[doc = "deno_bindgen"]
                impl CustomType {
                    fn some_fn() {}
                }
            }
        );
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl Item {
    pub fn transform(&mut self) {
        match self {
            Item::Fn(item_fn) => item_fn.transform(),
            Item::Impl(item_impl) => item_impl.transform(),
            _ => (), // do nothing for unsupported types
        }
    }
}

impl ToTokens for Item {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Item::Fn(item_fn) => item_fn.to_token_stream(),
            Item::Impl(item_impl) => item_impl.to_token_stream(),
            Item::Struct(item_struct) => item_struct.to_token_stream(),
            _ => TokenStream::new(), // do nothing for unsupported types
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[cfg(test)]
mod print_tests {}
