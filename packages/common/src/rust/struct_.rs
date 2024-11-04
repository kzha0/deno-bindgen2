use syn::token::Struct;

use crate::rust::util::*;
use crate::rust::Attribute;

// MARK: api

#[derive(Clone, Debug)]
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
        Ok(Self {
            attr,
            ident,
        })
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

impl ToTokens for ItemStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let self_ty = &self.ident;
        let ident = format_ident!("__{}__drop", self.ident);
        tokens.extend(quote! {
            impl ::deno_bindgen2::DenoBindgen for #self_ty {}
            #[unsafe(no_mangle)]
            extern "C" fn #ident (arg_0: *mut #self_ty) {
                ::deno_bindgen2::DenoBindgen::drop(arg_0);
            }
        });
    }
}
