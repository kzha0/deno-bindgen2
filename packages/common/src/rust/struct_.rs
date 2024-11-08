use crate::rust::util::*;
use crate::rust::Attribute;

// MARK: api

#[derive(Clone, Debug, PartialEq)]
pub struct ItemStruct {
    pub attr:  Attribute,
    pub ident: Ident,
}


impl Parse for ItemStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = Attribute::default();
        attr.parse_outer(input)?;
        input.parse::<Visibility>()?;

        input.parse::<Token![struct]>()?;
        Self::parse_remaining(input, attr)
    }
}

impl ItemStruct {
    pub fn parse_remaining(input: ParseStream, attr: Attribute) -> Result<Self> {
        // continued after parsing the `struct` token
        let ident = input.parse()?;

        // [!TODO] create helper for generics errors
        if let Some(lt_token) = input.parse::<Option<Token![<]>>()? {
            return Err(Error::new(
                lt_token.span(),
                "generic parameters are not supported",
            ));
        }

        if let Some(where_) = input.parse::<Option<Token![where]>>()? {
            return Err(Error::new(
                where_.span(),
                "generic parameters and where clauses are not supported",
            ));
        }

        let ahead = input.lookahead1();
        if ahead.peek(Paren) {
            syn::FieldsUnnamed::parse(input)?;
            let ahead = input.lookahead1();
            if ahead.peek(Token![;]) {
                input.parse::<Token![;]>()?;
            } else {
                return Err(ahead.error());
            }
        } else if ahead.peek(Brace) {
            syn::FieldsNamed::parse(input)?;
        } else if ahead.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        } else {
            return Err(ahead.error());
        }

        // fields are ignored for now. in the future, they may be supported directly
        Ok(Self { attr, ident })
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn test_parse_struct() {
        dbg_assert!(parse_quote!(ItemStruct, struct CustomType;), ItemStruct {
            attr:  Attribute::default(),
            ident: format_ident!("CustomType"),
        });
        dbg_assert!(
            parse_quote!(ItemStruct, struct CustomType(bool);),
            ItemStruct {
                attr:  Attribute::default(),
                ident: format_ident!("CustomType"),
            }
        );
        dbg_assert!(
            parse_quote!(
                ItemStruct,
                struct CustomType {
                    some_field: bool,
                }
            ),
            ItemStruct {
                attr:  Attribute::default(),
                ident: format_ident!("CustomType"),
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_struct_with_generics() {
        dbg_assert!(
            parse_quote!(ItemStruct, struct CustomType<T>(T);),
            ItemStruct {
                attr:  Attribute::default(),
                ident: format_ident!("CustomType"),
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_struct_with_where_clause() {
        dbg_assert!(
            parse_quote!(
                ItemStruct,
                struct CustomType
                where
                    T: Sized,
                {
                    some_field: bool,
                }
            ),
            ItemStruct {
                attr:  Attribute::default(),
                ident: format_ident!("CustomType"),
            }
        );
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl ToTokens for ItemStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let self_ty = &self.ident;
        let ident = format_ident!("__{}__drop", self.ident);
        tokens.extend(quote! {
            impl deno_bindgen2::DenoBindgen for #self_ty {}
            #[unsafe(no_mangle)]
            extern "C" fn #ident (arg_0: *mut #self_ty) {
                std::mem::drop(Box::from(arg_0));
            }
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[cfg(test)]
mod print_tests {
    use super::*;

    #[test]
    fn test_print_struct() {
        let raw = parse_quote!(
            ItemStruct,
            pub struct CustomType {
                field: bool,
            }
        );
        println!(
            "{}",
            crate::prettify!(raw.to_token_stream().to_string().as_str())
        );
    }
}
