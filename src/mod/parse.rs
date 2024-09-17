use darling::{
    ast::NestedMeta,
    FromMeta,
};
use proc_macro_error::{abort, emit_error};
use proc_macro2::TokenStream;
use quote::ToTokens;

pub mod r#fn;

pub use crate::r#mod::*;
pub use r#fn::*;

#[derive(Default, FromMeta, Debug)]
#[darling(default)]
pub struct MacroArgs {
    optional:     bool,
    non_blocking: bool,
}

pub struct Parser();
impl Parser {
    pub fn from_attr_stream(
        attr: proc_macro::TokenStream,
        input: proc_macro::TokenStream
    ) -> proc_macro::TokenStream {
        let macro_args = NestedMeta::parse_meta_list(attr.into())
            .and_then(|meta_list| {
                let mut this = MacroArgs::default();
                for meta in meta_list {
                    let other =
                        MacroArgs::from_list(&*Box::new([meta.clone()])).unwrap_or_else(|err| {
                            abort!(meta, "failed to parse macro attribute: {}", err)
                        });
                    this = MacroArgs {
                        optional:     this.optional | other.optional,
                        non_blocking: this.non_blocking | other.non_blocking,
                    };
                }
                Ok(this)
            })
            .unwrap_or_else(|err| {
                abort!("failed to parse attribute stream: {}", err);
            });

        let item: Item = syn::parse2(input.into()).unwrap_or_else(|err| {
            abort!("failed to parse input code: {}", err)
        });

        parse_item(item, macro_args).into()
    }
}

fn parse_item(item: Item, macro_args: MacroArgs) -> TokenStream {
    match item {
        Item::Fn(item_fn) => {
            parse_fn(
                item_fn,
                FnArgs {
                    optional:     macro_args.optional,
                    non_blocking: macro_args.non_blocking,
                    _internal:    false,
                },
            )
        },
        Item::Struct(_) => todo!(),
        Item::Impl(_) => todo!(),

        item => {
            emit_error!(
                item, "unsupported AST item";
                note = "`deno_bindgen` may only be used with `fn`, `struct`, and `impl` code.";
            );
            item.to_token_stream()
        },
    }
}
