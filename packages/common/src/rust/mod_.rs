use crate::rust::util::*;
pub use crate::rust::{Attribute, Item, ItemFn, ItemImpl};

/* -------------------------------------------------------------------------- */

// MARK: api

#[derive(Clone, Debug)]
pub struct ItemMod {
    pub attr:  Attribute,
    pub ident: Ident,
    pub items: Vec<Item>,
}

impl Parse for ItemMod {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = Attribute::default();
        attr.parse_outer(input)?;
        input.parse::<Visibility>()?;

        input.parse::<Option<Token![unsafe]>>()?;
        input.parse::<Token![mod]>()?;

        Self::parse_remaining(input, attr)
    }
}

impl ItemMod {
    pub fn parse_remaining(input: ParseStream, mut attr: Attribute) -> Result<Self> {
        let ident: Ident = if input.peek(Token![try]) {
            input.call(Ident::parse_any)
        } else {
            input.parse::<Ident>()
        }?;

        let mut items = Vec::new();

        let ahead = input.lookahead1();
        if ahead.peek(Token![;]) {
            input.parse::<Token![;]>()?;
        } else if ahead.peek(Brace) {
            let content;
            braced!(content in input);
            attr.parse_inner(&content)?;

            items.append(&mut Self::parse_content(&content)?);
        } else {
            return Err(ahead.error());
        }

        Ok(Self { attr, ident, items })
    }

    pub fn parse_content(input: ParseStream) -> Result<Vec<Item>> {
        let mut items = Vec::new();

        while !input.is_empty() {
            let item = Item::parse(input)?;
            // filter only `deno_bindgen` items
            match &item {
                Item::Fn(ItemFn { attr, .. }) | Item::Impl(ItemImpl { attr, .. }) => {
                    if attr.has_deno_bindgen() {
                        items.push(item)
                    }
                },
                Item::Mod(_) => items.push(item),
                _ => (),
            }
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod() {
        dbg_quote!(
            ItemMod,
            #[this_mod]
            #[doc = "document this mod"]
            mod my_mod {
                #![doc = "inner docs"]

                #[doc = "deno_bindgen"]
                #[doc = "some_documentation"]
                fn my_fn() {
                    #![doc = "inner docs"]
                }

                fn ignored_function() {}

                struct CustomType {}

                #[doc = "deno_bindgen"]
                impl CustomType {
                    #![doc = "inner docs"]

                    fn some_fn() {}
                }
            }
        );
    }
}
