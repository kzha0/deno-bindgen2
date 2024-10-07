use proc_macro2::{
    Ident,
    TokenStream,
};
use proc_macro_error::emit_error;
use quote::{
    format_ident,
    quote,
};
use syn::{
    parse_quote,
    FnArg,
    ItemFn,
    ItemImpl,
    PatType,
    ReturnType,
    Type,
    TypePath,
    TypeReference,
};

pub use crate::*;

pub fn parse_impl(item_impl: &ItemImpl) -> Result<TokenStream> {

    //-------------------------------- CHECKS ------------------------------/

    if let Some(generic) = item_impl.generics.params.first() {
        emit_error!(generic, "generics are not supported");
        return Err(());
    };
    if let Some(where_clause) = &item_impl.generics.where_clause {
        emit_error!(where_clause, "where clauses are not supported");
        return Err(());
    };

    //-------------------------------- PARSER ------------------------------/

    fn get_arg_idents(args: Vec<FnArg>) -> Vec<syn::Ident> {
        args.iter()
            .map(|arg| {
                match arg {
                    FnArg::Receiver(_) => unreachable!(),
                    FnArg::Typed(PatType { pat, .. }) => {
                        match &**pat {
                            syn::Pat::Ident(ident) => &ident.ident,
                            _ => unreachable!(),
                        }
                    },
                }
            })
            .cloned()
            .collect()
    }

    let impl_ty = match *item_impl.self_ty {
        syn::Type::Path(TypePath { ref path, .. }) => path.get_ident().ok_or(())?,
        _ => {
            emit_error!(*item_impl.self_ty, "unsupported type");
            return Err(());
        },
    };

    let mut out_fns: Vec<TokenStream> = Vec::new();
    let mut raw_fns: Vec<RawFnBuilder> = Vec::new();

    for item in item_impl.items.iter() {
        if let syn::ImplItem::Fn(syn::ImplItemFn { sig, attrs, .. }) = &item {
            let _constructor: bool;
            if let Some(attr) = attrs.first() {
                _constructor = attr.path().is_ident("constructor");
            } else {
                _constructor = false;
            };

            let raw_method_name = &sig.ident;
            let method_name = format_ident!("{}_{}", impl_ty, sig.ident);
            let is_self_output = || {
                fn match_ty(ty: &Box<Type>, impl_ty: &Ident) -> Option<Ident> {
                    match &**ty {
                        syn::Type::Path(TypePath { ref path, .. }) => {
                            let this_ident = path.get_ident()?;
                            match this_ident.to_string().as_str() {
                                "Self" => Some(impl_ty.clone()),
                                _ => {
                                    if impl_ty == path.get_ident()? {
                                        Some(impl_ty.clone())
                                    } else {
                                        None
                                    }
                                },
                            }
                        },
                        syn::Type::Reference(TypeReference { elem, .. }) => {
                            match_ty(&elem, impl_ty)
                        },
                        _ => None,
                    }
                }

                match &sig.output {
                    syn::ReturnType::Default => None,
                    syn::ReturnType::Type(_, ty) => match_ty(ty, impl_ty),
                }
            };

            // Has a self type argument
            let item_fn: ItemFn = if sig.receiver().is_some() {
                let mut inputs: Vec<FnArg> = sig.inputs.iter().skip(1).cloned().collect();
                let arg_idents: Vec<Ident> = get_arg_idents(inputs.clone());
                inputs.push(parse_quote!(self_: #impl_ty));

                let output = if let Some(output) = is_self_output() {
                    &parse_quote!(-> #output)
                } else {
                    &sig.output.clone()
                };

                parse_quote!(
                    #[allow(non_snake_case)]
                    fn #method_name (#(#inputs),*) #output {
                        // let self_ = unsafe { &mut *self_ };
                        self_. #raw_method_name (#(#arg_idents),*)
                    }
                )

            // Creates a self type
            } else if _constructor {
                let inputs: Vec<FnArg> = sig.inputs.iter().cloned().collect();
                let arg_idents = get_arg_idents(inputs.clone());
                let output: &ReturnType = if let Some(output) = is_self_output() {
                    &parse_quote!(-> #output)
                } else {
                    emit_error!(&sig.output, "constructor methods must return a `Self` type or the associated struct type");
                    return Err(());
                };

                parse_quote!(
                    #[allow(non_snake_case)]
                    fn #method_name (#(#inputs),*) #output {
                        #impl_ty:: #raw_method_name (#(#arg_idents),*)
                    }
                )
            } else {
                let inputs: Vec<FnArg> = sig.inputs.iter().cloned().collect();
                let arg_idents = get_arg_idents(inputs.clone());
                let output = if let Some(output) = is_self_output() {
                    &parse_quote!(-> #output)
                } else {
                    &sig.output.clone()
                };

                parse_quote!(
                    #[allow(non_snake_case)]
                    fn #method_name (#(#inputs),*) #output {
                        #impl_ty:: #raw_method_name (#(#arg_idents),*)
                    }
                )
            };

            let (out, mut raw_fn) = parse_fn(&item_fn, MacroArgsFn {
                _internal: true,
                _constructor,
                ..Default::default()
            })?;

            raw_fn.ident = format_ident!("_{}", method_name);
            out_fns.push(out);
            raw_fns.push(raw_fn);
        };
    }

    Ok(quote! {
        #item_impl
        #(#out_fns)*
        const _: () = {#[deno_bindgen2::linkme::distributed_slice(deno_bindgen2::RAW_ITEMS)]
            pub static __: deno_bindgen2::RawItem = deno_bindgen2::RawItem::Struct(deno_bindgen2::RawStruct{
                ident: stringify!(#impl_ty),
                methods: &[#(#raw_fns),*]
            });
        };
    })
}
