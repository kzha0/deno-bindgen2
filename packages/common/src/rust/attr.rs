use crate::rust::util::*;
use crate::rust::Item;

/* -------------------------------------------------------------------------- */

// MARK: marker

/// `markers` here denote live attributes that get converted to inert attributes
/// (as doc attributes) the live version is used by the proc macro library,
/// while the inert versions (the doc attributes ones) are used by the cli to
/// control code generation. it does this by using the parser implementation
/// here to read the marker info
#[derive(Clone, Debug, PartialEq)]
pub enum Marker {
    DenoBindgen, // marks a deno bindgen item. automatically inserted by the item macro
    NonBlocking, /* marks a function as non-blocking */

                 /* [!TODO] support for translating member visibility https://www.typescriptlang.org/docs/handbook/2/classes.html#member-visibility
                  * interpret visibility of rust functions and interpolate as class visibility
                  * useful for implementing internal methods */
}

// TODO: move from doc attributes to inert attributes
// support custom inert attributes rfc
// https://github.com/rust-lang/rust/issues/66079

#[cfg(feature = "macro")]
impl Marker {
    pub fn deno_bindgen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
        let input = TokenStream::from(input);
        let mut item: Item = match syn::parse2(input.clone()) {
            Ok(item) => item,
            Err(err) => return err.to_compile_error().into(),
        };
        item.transform();
        quote! {
            #[cfg_attr(not(deno_bindgen), doc = "deno_bindgen")]
            #input
            #item
        }
        .into()
    }

    pub fn non_blocking(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
        let input = TokenStream::from(input);
        quote! {
            #[cfg_attr(not(deno_bindgen), doc = "deno_bindgen_non_blocking")]
            #input
        }
        .into()
    }
}

/* -------------------------------------------------------------------------- */

// MARK: meta

/// a document attribute meta in the form `doc = "value"`
#[derive(Clone, Debug, PartialEq)]
pub struct Meta {
    pub lit_str: LitStr,
}

impl TryFrom<&Meta> for Marker {
    fn try_from(value: &Meta) -> Result<Self> {
        match value.lit_str.value().as_str() {
            "deno_bindgen" => Ok(Self::DenoBindgen),
            "deno_bindgen_non_blocking" => Ok(Self::NonBlocking),
            _ => Err(Error::new(
                value.lit_str.span(),
                "unknown value. expected one of `deno_bindgen`, `deno_bindgen_non_blocking`, `deno_bindgen_constructor`"
            )),
        }
    }

    type Error = Error;
}

impl Parse for Meta {
    fn parse(input: ParseStream) -> Result<Self> {
        let key = input.parse::<Ident>()?;

        if input.is_empty() {
            let key_str = key.to_string();
            match key_str.as_str() {
                "non_blocking" => {
                    let lit_str = LitStr::new(
                        format!("deno_bindgen_{key_str}").as_str(),
                        Span::mixed_site(),
                    );
                    return Ok(Self { lit_str });
                },
                _ => (),
            }
        }

        input.parse::<Token![=]>()?;
        if key.to_string().as_str() == "doc" {
            let lit_str = input.parse()?;

            // emit error if there are remaining tokens in the stream
            if !input.is_empty() {
                Err(input.error("unknown token"))
            } else {
                Ok(Self { lit_str })
            }
        } else {
            Err(Error::new(key.span(), "expected `doc` key"))
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: attribute

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attribute {
    pub markers: Vec<Marker>,
    pub meta:    Vec<Meta>,
    // pub doc: Vec<String>, // [!TODO] support for documentation in code, with auto-generated docs
    // by the tool
}

impl Attribute {
    /// checks if this attribute contains the `deno_bindgen` marker
    pub fn has_deno_bindgen(&self) -> bool {
        self.markers
            .iter()
            .find(|marker| match marker {
                Marker::DenoBindgen => true,
                _ => false,
            })
            .is_some()
    }

    /// checks if this attribute contains the `non_blocking` marker
    pub fn has_non_blocking(&self) -> bool {
        self.markers
            .iter()
            .find(|marker| match marker {
                Marker::NonBlocking => true,
                _ => false,
            })
            .is_some()
    }
}

impl Attribute {
    pub fn parse_outer(&mut self, input: ParseStream) -> Result<()> {
        while input.peek(Token![#]) {
            input.parse::<Token![#]>()?;
            if input.peek(Token![!]) {
                return Err(Error::new(
                    input.span(),
                    "attempted to parse inner attribute in a parser for outer attributes",
                ));
            }

            let content;
            bracketed!(content in input);

            let fork = content.fork();
            if let Ok(meta) = fork.parse::<Meta>() {
                content.advance_to(&fork);
                if let Ok(marker) = Marker::try_from(&meta) {
                    self.markers.push(marker);
                } else {
                    self.meta.push(meta);
                }
            } else {
                // content should have been exhausted by doc_meta parser
                content.parse::<syn::Meta>()?;
            }
        }

        Ok(())
    }

    pub fn parse_inner(&mut self, input: ParseStream) -> Result<()> {
        while input.peek(Token![#]) && input.peek2(Token![!]) {
            input.parse::<Token![#]>()?;
            input.parse::<Token![!]>()?;
            let content;
            bracketed!(content in input);

            let fork = content.fork();
            if let Ok(meta) = fork.parse::<Meta>() {
                content.advance_to(&fork);
                self.meta.push(meta);
            } else {
                content.parse::<syn::Meta>()?;
            }
        }

        Ok(())
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod tests {
    use super::*;

    impl Parse for Attribute {
        /// Used for debugging only. Cannot mix inner or outer attributes with
        /// each other in a single parse run
        fn parse(input: ParseStream) -> Result<Self> {
            let mut attr = Self::default();

            let ahead = input.lookahead1();
            if ahead.peek(Token![#]) {
                if input.peek2(Token![!]) {
                    attr.parse_inner(input)?;
                } else {
                    attr.parse_outer(input)?;
                }
            } else {
                return Err(ahead.error());
            }

            Ok(attr)
        }
    }

    #[test]
    fn test_attr() {
        dbg_quote!(Attribute,
            #[some_attr]
            #[doc = "some unknown value"]
            #[another_attr]
        );
    }

    #[test]
    #[should_panic]
    fn test_mix_attr() {
        dbg_quote!(Attribute,
            #[outer]
            #![innter]
        );
    }

    #[test]
    fn test_marker() {
        dbg_quote!(Attribute,
            #[doc = "deno_bindgen_constructor"]
            #[doc = "deno_bindgen"]
        );
    }

    #[test]
    fn test_cfg_attr() {
        dbg_quote!(Attribute,
            #[cfg_attr(not(deno_bindgen), doc = "deno_bindgen_constructor")]
        );
    }

    #[test]
    fn test_live_attr() {
        dbg_quote!(Attribute,
            #[constructor]
        );
    }
}
