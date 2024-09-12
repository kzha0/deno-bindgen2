use proc_macro2::{
    Ident,
    Span,
    TokenStream,
};
use proc_macro_error::{
    abort,
    emit_error,
};
use quote::{
    quote,
    ToTokens, TokenStreamExt,
};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    Expr,
    ExprCall,
    ExprTuple,
    FnArg,
    ItemFn,
    PatType,
};

use crate::r#mod::*;

pub struct Transformer();

pub struct TransformItem(pub Item);
impl TransformItem {
    pub fn transform(self) -> proc_macro::TokenStream {
        let TransformItem(item) = self;
        match item {
            Item::Fn(item) => TransformFnItem::transform(&item),
            _ => todo!(),
        }
    }
}

pub struct TransformFnItem();
impl TransformFnItem {
    pub fn transform(item: &ItemFn) -> proc_macro::TokenStream {
        // INPUTS
        let raw_inputs: Vec<IrType> = item
            .sig
            .inputs
            .iter()
            .map(|fn_arg| {
                match fn_arg {
                    FnArg::Receiver(receiver) => {
                        IrType::NonTrivial(NonTrivial::Receiver(receiver.clone()))
                    },
                    FnArg::Typed(PatType { ty, .. }) => IrType::from(*ty.clone()),
                }
            })
            .collect();

        let mut inputs: Punctuated<FnArg, Comma> = Punctuated::new();
        let mut call_args: Punctuated<Expr, Comma> = Punctuated::new();

        for ty in raw_inputs.iter() {
            let index = inputs.len();
            call_args.push(expand_inputs(index, ty, &mut inputs));
        };


        // SIGNATURE
        let ident = item.sig.ident.clone();
        let mut out_fn: ItemFn = parse_quote!(
            #[no_mangle]
            extern "C" fn #ident() {

            }
        );

        out_fn.sig.inputs = inputs;
        out_fn.sig.output = item.sig.output.clone();

        let mut fn_call: ExprCall = parse_quote!(#ident());
        fn_call.args = call_args;
        out_fn.block = parse_quote!({
            #item
            #fn_call
        });

        out_fn.to_token_stream().into()
    }
}

// this is for inputs
fn expand_inputs(pos: usize, ty: &IrType, inputs: &mut Punctuated<FnArg, Comma>) -> Expr {
    match ty {
        IrType::Native(ty) => {
            let ident = Ident::new(&format!("arg{}", pos), Span::mixed_site());
            inputs.push(parse_quote!(#ident: #ty));

            parse_quote!(#ident)
        },
        IrType::NonTrivial(ty) => {
            match ty {
                NonTrivial::Tuple(tys) => {
                    // triggers if this function was called recursively
                    let mut tup: ExprTuple = parse_quote!( () );
                    for (index, ty) in tys.iter().enumerate() {
                        let expr = expand_inputs(pos + index, ty, inputs);
                        tup.elems.push(expr);
                    }

                    parse_quote!(#tup)
                },
                NonTrivial::Slice(ty) => {
                    let ptr = Ident::new(&format!("arg{}", pos), Span::mixed_site());
                    let len = Ident::new(&format!("arg{}", pos + 1), Span::mixed_site());

                    let elem: Expr;
                    match *ty.clone() {
                        IrType::Native(ty) => {
                            if ty.is_numeric() {
                                elem = parse_quote!(Native::Buffer(Box::new(ty)));
                            } else {
                                elem = parse_quote!(Native::Pointer);
                            }
                        },
                        IrType::NonTrivial(_) => {
                            elem = expand_inputs(pos, &ty, inputs);
                        },
                        IrType::Unsupported(_) => todo!(),
                    }
                    inputs.push(parse_quote!(#ptr: #elem));
                    inputs.push(parse_quote!(#len: usize));
                    parse_quote!(unsafe { std::slice::from_raw_parts(#ptr as #elem, #len as usize) })
                },
                NonTrivial::StringSlice => {
                    let ptr = Ident::new(&format!("arg{}", pos), Span::mixed_site());
                    let len = Ident::new(&format!("arg{}", pos + 1), Span::mixed_site());

                    inputs.push(parse_quote!(#ptr: *const u8));
                    inputs.push(parse_quote!(#len: usize));
                    parse_quote!(unsafe { std::str::from_raw_parts(#ptr as *const u8, #len as usize) })
                },
                _ => todo!("cannot transform non-trivial type"),
            }
        },
        _ => todo!("cannot transform unknown type"),
    }
}

impl ToTokens for Native {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ty = match self {
            Native::Void => quote! { () },
            Native::Bool => quote! { bool },
            Native::U8 => quote! { u8 },
            Native::I8 => quote! { i8 },
            Native::U16 => quote! { u16 },
            Native::I16 => quote! { i16 },
            Native::U32 => quote! { u32 },
            Native::I32 => quote! { i32 },
            Native::U64 => quote! { u64 },
            Native::I64 => quote! { i64 },
            Native::USize => quote! { usize },
            Native::ISize => quote! { isize },
            Native::F32 => quote! { f32 },
            Native::F64 => quote! { f64 },
            Native::Pointer => quote! { *const () },
            Native::Function => quote! { *const fn },
            Native::Buffer(ty) => {
                let ty = *ty.clone();
                quote! { *const #ty }
            },
        };

        tokens.extend(ty);
    }
}
