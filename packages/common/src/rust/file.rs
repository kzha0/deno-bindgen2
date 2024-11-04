use crate::rust::util::*;
use crate::rust::{Attribute, Item, ItemMod};

/* -------------------------------------------------------------------------- */

// MARK: file api

#[derive(Clone, Debug)]
pub struct File {
    pub attr:  Attribute,
    pub items: Vec<Item>,
}

impl Parse for File {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = Attribute::default();
        attr.parse_inner(input)?;
        let items = ItemMod::parse_content(input)?;

        Ok(Self { attr, items })
    }
}

impl File {
    pub fn parse_str(mut content: &str) -> Self {
        const BOM: &str = "\u{feff}";
        if content.starts_with(BOM) {
            content = &content[BOM.len()..];
        }

        // let mut shebang = None;
        // if content.starts_with("#!") {
        //     let rest = whitespace::skip(&content[2..]);
        //     if !rest.starts_with('[') {
        //         if let Some(idx) = content.find('\n') {
        //             shebang = Some(content[..idx].to_string());
        //             content = &content[idx..];
        //         } else {
        //             shebang = Some(content.to_string());
        //             content = "";
        //         }
        //     }
        // }

        syn::parse_str(content).expect("failed to parse file")
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    #[test]
    fn test_item_fn() {
        let content = quote! {

            const SOME_CONST: &'static str = "Hello, World!";

            fn some_item() {}

            #[doc = "deno_bindgen"]
            fn some_item_annotated() {}

            struct CustomType {}

            #[doc = "deno_bindgen"]
            impl CustomType {
                pub fn some_fn(string: String) {}
            }

            mod SomeMod {}

        }
        .to_string();

        let content = File::parse_str(content.as_str());
        dbg!(content);
    }
}
