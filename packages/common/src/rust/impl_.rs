use crate::rust::{Attribute, ItemFn};
use crate::rust::util::*;

/* -------------------------------------------------------------------------- */

// MARK: impl api

#[derive(Clone, Debug)]
pub struct ItemImpl {
    pub attr:    Attribute,
    pub unsafe_: Option<Token![unsafe]>,
    pub self_ty: Ident,
    pub items:   Vec<ItemFn>,
}

// MARK: parse

impl Parse for ItemImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = Attribute::default();
        attr.parse_outer(input)?;
        input.parse::<Visibility>()?;

        let unsafe_ = input.parse()?;
        input.parse::<Token![impl]>()?;
        ItemImpl::parse_remaining(input, attr, unsafe_)
    }
}

impl ItemImpl {
    /// `parse_with_progress` is used by the `item` parser since it has already
    /// parsed some attribute or visibility for both item_fn and item_impl
    pub fn parse_remaining(
        input: ParseStream,
        mut attr: Attribute,
        // visibility already parsed
        unsafe_: Option<Token![unsafe]>,
    ) -> Result<Self> {
        // continued after parsing the `impl` token
        if let Some(lt_token) = input.parse::<Option<Token![<]>>()? {
            return Err(Error::new(
                lt_token.span(),
                "generic parameters are not supported",
            ));
        }

        let fork = input.fork();

        // is the leading colon okay if the path is global? this may refer to an item
        // outside the current crate
        let leading_colon = fork.parse::<Option<Token![::]>>()?;
        let ident = fork.parse::<Ident>();
        let path_sep = fork.parse::<Option<Token![::]>>()?;
        let lt_token = fork.parse::<Option<Token![<]>>()?;

        if let Some(lt_token) = lt_token {
            // [!ISSUE] implementing types cannot have generic parameters as it leads to
            // multiple polymorphic implementations of the same type method
            return Err(Error::new(
                lt_token.span(),
                "type arguments are not supported",
            ));
        }
        if let Some(leading_colon) = leading_colon {
            return Err(Error::new(leading_colon.span(), "unsupported global path"));
        }
        if let Some(path_sep) = path_sep {
            return Err(Error::new(
                path_sep.span(),
                "unsupported type path: only bare identifiers are supported with no path segments `::` nor type arguments. bring the type into scope with a `use` statement"
            ));
        }

        let self_ty;
        // try to get the `Ident`
        if let Ok(ident) = ident {
            input.advance_to(&fork);
            self_ty = ident
        } else {
            // try to parse as syn::Type to give the user more info
            let ty = input.call(syn::Type::parse);
            if let Ok(ty) = ty {
                return Err(Error::new(
                    ty.span(),
                    "unsupported type: only bare identifiers are supported",
                ));
            } else {
                return Err(ident.unwrap_err());
            }
        };

        if let Some(for_) = input.parse::<Option<Token![for]>>()? {
            return Err(Error::new(
                for_.span(),
                "unexpected token: trait implements are not supported",
            ));
        }

        if let Some(where_) = input.parse::<Option<Token![where]>>()? {
            return Err(Error::new(
                where_.span(),
                "generic parameters and where clauses are not supported",
            ));
        }

        let content;
        braced!(content in input);
        attr.parse_inner(&content)?;

        let mut items = Vec::new();
        let mut constructor = false; // tracks a singleton `constructor` marker

        while !content.is_empty() {
            let mut attr = Attribute::default();
            attr.parse_outer(&content)?;
            content.parse::<Visibility>()?;

            let fork = content.fork();
            let item = ItemFn::parse_self_ty(&fork, attr, Some(&self_ty));
            if let Ok(item) = item {
                if item.attr.has_constructor() {
                    if !constructor {
                        constructor = true;
                    } else {
                        return Err(Error::new(item.ident.span(), "`constructor` marker used multiple times. only one function can be marked with `constructor`"));
                    }
                }

                content.advance_to(&fork);
                items.push(item);
            } else {
                let syn_item = content.call(syn::ImplItem::parse)?;
                match syn_item {
                    syn::ImplItem::Fn(_) => {
                        return Err(item.unwrap_err());
                    },
                    _ => (),
                };
            }
        }

        Ok(Self {
            attr,
            unsafe_,
            self_ty,
            items,
        })
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn with_attrs_and_vis() {
        dbg_quote!(
            ItemImpl,
            #[some_attr]
            pub impl CustomType {}
        );
    }

    #[test]
    fn with_unsafe() {
        dbg_quote!(ItemImpl, unsafe impl CustomType {});
    }

    #[test]
    #[should_panic]
    fn with_generics() {
        dbg_quote!(ItemImpl, impl<T> CustomType {});
    }

    #[test]
    fn with_empty() {
        dbg_quote!(ItemImpl, impl CustomType {});
    }

    #[test]
    #[should_panic]
    fn with_for() {
        dbg_quote!(ItemImpl, impl Debug for CustomType {});
    }

    #[test]
    #[should_panic]
    fn with_global_path() {
        dbg_quote!(ItemImpl,
            impl ::CustomType {

            }
        );
    }

    #[test]
    #[should_panic]
    fn with_path() {
        dbg_quote!(ItemImpl,
            impl my_mod::CustomType {

            }
        );
    }

    #[test]
    #[should_panic]
    fn with_where() {
        dbg_quote!(ItemImpl,
            impl CustomType
            where
                T: Drop
            {

            }
        );
    }

    #[test]
    fn with_item_fn() {
        dbg_quote!(ItemImpl,
            impl CustomType {
                fn test_fn() {}
            }
        );
    }

    #[test]
    fn with_nested_attrs_and_vis() {
        dbg_quote!(
            ItemImpl,
            #[impl_outer_attr]
            impl CustomType {
                #![impl_inner_attr]

                #[fn_outer_attr]
                pub fn test_fn() {
                    #![fn_inner_attr]
                }
            }
        );
    }

    #[test]
    fn with_many() {
        dbg_quote!(ItemImpl,
            impl CustomType {
                fn test_fn() {}

                fn test_fn2() {}

                fn test_fn3() {}
            }
        );
    }

    #[test]
    #[should_panic]
    fn with_self() {
        dbg_quote!(ItemImpl,
            impl CustomType {
                fn test_fn(self) {}
            }
        );
    }

    #[test]
    fn with_unsafe_self() {
        dbg_quote!(
            ItemImpl,
            unsafe impl CustomType {
                unsafe fn test_fn(self) {}
            }
        );
    }

    #[test]
    fn with_self_ref() {
        dbg_quote!(ItemImpl,
            impl CustomType {
                fn test_fn(&mut self) {}
            }
        );
    }

    #[test]
    fn with_other_self() {
        dbg_quote!(ItemImpl,
            impl CustomType {

                fn test_fn(arg0: Self) {}
            }
        );
    }

    #[test]
    fn with_other_selves() {
        dbg_quote!(ItemImpl,
            impl CustomType {
                fn test_fn(
                    &mut self,
                    arg0: (Vec<Self>, &mut Self),
                    arg3: Box<Self>
                ) -> Box<Self> {}
            }
        );
    }

    #[test]
    fn with_other_items() {
        // ignores non-fn items
        dbg_quote!(ItemImpl,
            impl CustomType {
                fn test_fn() {}

                type Some = usize;

                fn test_fn2() {}

                const SOME_STR: &str = "Str";
            }
        );
    }

    #[test]
    fn with_constructor() {
        dbg_quote!(ItemImpl,
            impl CustomType {

                #[constructor]
                fn test_fn() -> Self {}

            }
        );
    }

    #[test]
    #[should_panic]
    fn with_constructor_error() {
        dbg_quote!(ItemImpl,
            impl CustomType {

                #[constructor]
                fn test_fn() -> OtherType {}

            }
        );
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl ItemImpl {
    pub fn transform(&mut self) {
        for item in &mut self.items {
            item.transform();
        }
    }
}

impl ToTokens for ItemImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let self_ty = &self.self_ty;
        let items = &self.items;
        tokens.extend(quote! {
            #(#items)*
            const _: () = {
                const fn assert_impl<T: ::deno_bindgen2::DenoBindgen>() {}
                assert_impl::<#self_ty>();
            };
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[cfg(test)]
mod print_tests {}
