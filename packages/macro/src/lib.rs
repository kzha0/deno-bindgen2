use darling::{
    ast::NestedMeta,
    FromMeta,
};
use proc_macro::TokenStream;
use proc_macro_error::{
    abort,
    emit_error,
};
use quote::{quote, ToTokens};
use syn::Item;

mod r#fn;
mod r#impl;
mod ir;
mod util;
use ir::*;
use r#fn::*;
use r#impl::*;
use util::*;

/*========================================================================*/

#[derive(Default, FromMeta, Debug)]
#[darling(default)]
struct MacroArgs {
    optional:     bool,
    non_blocking: bool,
}

#[proc_macro_attribute]
#[proc_macro_error::proc_macro_error]
pub fn deno_bindgen(attr: TokenStream, input: TokenStream) -> TokenStream {
    //-------------------------------- CHECKS ------------------------------/

    let macro_args = NestedMeta::parse_meta_list(attr.into())
        .and_then(|meta_list| {
            let mut this = MacroArgs::default();
            for meta in meta_list {
                let other = MacroArgs::from_list(&*Box::new([meta.clone()]))
                    .unwrap_or_else(|err| abort!(meta, "failed to parse macro attribute: {}", err));
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

    let item: Item = syn::parse2(input.into())
        .unwrap_or_else(|err| abort!("failed to parse input code: {}", err));

    //-------------------------------- PARSER ------------------------------/

    match item {
        Item::Fn(item_fn) => {
            parse_fn(&item_fn, MacroArgsFn {
                non_blocking: macro_args.non_blocking,
                _internal:    false,
                _constructor: false,
            })
                .and_then(|(mut out, raw_fn)| {
                    out.extend(quote! {
                        const _: () = {
                            #[deno_bindgen2::linkme::distributed_slice(deno_bindgen2::RAW_ITEMS)]
                            pub static __: deno_bindgen2::RawItem = deno_bindgen2::RawItem::Fn(#raw_fn);
                        };
                    });
                    Ok(out)
                })
                .unwrap_or(item_fn.to_token_stream())
        },
        Item::Impl(item_impl) => {
            parse_impl(&item_impl)
                .unwrap_or(item_impl.to_token_stream())
        },
        _ => {
            emit_error!(
                item, "unsupported AST item";
                note = "`deno_bindgen` may only be used with `fn`, `struct`, and `impl` code.";
            );
            item.to_token_stream()
        },
    }
    .into()
}
