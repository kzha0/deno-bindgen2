use proc_macro2::{TokenStream, Ident, Span};
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, Expr, ExprCall, ExprTuple, FnArg, ItemFn, PatType, ReturnType, Signature, Stmt, Type, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple};

pub use crate::r#mod::*;

pub struct FnArgs {
    pub optional:     bool,
    pub non_blocking: bool,
    pub _internal:    bool,
    pub _constructor: bool,
}

pub fn parse_fn(item_fn: ItemFn, fn_args: FnArgs) -> TokenStream {

    // Prequalifying checks
    match item_fn.sig {
        Signature { asyncness, .. } if asyncness.is_some() => {
            emit_error!(Some(asyncness), "async functions are unsupported");
        },
        Signature { ref abi, .. } if (abi.is_some()) => {
            match &abi.as_ref().unwrap().name {
                // return early if ABI string is "C"
                Some(name) if name.value() == "C".to_string() => (),
                Some(name) => {
                    emit_error!(
                        name, "unsupported ABI string: \"{}\"", name.value();
                        note = "Deno's FFI API supports only the C ABI";
                        help = "change the ABI string to \"C\" or remove \"{}\"", name.value();
                    );
                },
                // this match arm triggers when there is no explicit ABI string
                // an `extern` qualifier without an explicit ABI string defaults to the "C" ABI
                // (source)[https://doc.rust-lang.org/reference/items/functions.html#extern-function-qualifier]
                None => {
                    emit_warning!(
                        abi, "unnecessary `extern` qualifier";
                        note = "`deno_bindgen2` handles insertion of ABI qualifiers automatically";
                        help = "remove the `extern` qualifier"
                    );
                },
            }
        },
        Signature { .. } => (),
    };


    parse_raw_fn(&item_fn, fn_args).unwrap_or(item_fn.to_token_stream())
}

// MARK: Collect transforms

fn parse_raw_fn(item_fn: &ItemFn, fn_args: FnArgs) -> Result<TokenStream> {


    let mut extern_fn = ExternFn::default()
        .with_ident(&item_fn.sig.ident);

    let mut in_args: Vec<FnArg> = Vec::new();
    let mut in_stmts: Vec<Stmt> = Vec::new();
    let mut out_stmts: Vec<Stmt> = Vec::new();
    let mut out_fns: Vec<ItemFn> = Vec::new();

    let raw_fn = RawFnBuilder::default()
        .with_ident(&extern_fn.0.sig.ident)
        .with_output({
            match &item_fn.sig.output {
                ReturnType::Default => {
                    IrType::default()
                },
                ReturnType::Type(_, ty) => {
                    let (ty, ir_ty) = parse_output(
                        &*ty,
                        &extern_fn.0.sig.ident.to_string().as_str(),
                        &mut out_stmts,
                        &mut out_fns,
                    )?;

                    extern_fn.with_output(parse_quote!(-> #ty), &mut out_stmts);
                    ir_ty
                },
            }
        })
        .with_inputs({
            let mut exprs: Vec<Expr> = Vec::new();
            let mut ir_tys: Vec<IrType> = Vec::new();

            for fn_arg in &item_fn.sig.inputs {
                match fn_arg {
                    FnArg::Receiver(receiver) => {
                        emit_error!(fn_arg, "unsupported self type");
                        return Err(());
                    },
                    FnArg::Typed(PatType{ ty, .. }) => {
                        let (expr, ir_ty) = parse_input(
                            &*ty,
                            &mut in_stmts,
                            &mut in_args
                        )?;
                        exprs.push(expr);
                        ir_tys.push(ir_ty);
                    },
                }
            };

            extern_fn.with_call(&item_fn.sig.ident, exprs);
            extern_fn.with_inputs(in_args, in_stmts);
            ir_tys
        });


    // contents of token stream output
    // - struct with slice
    // - original fn
    // - extern fns
    let mut out = quote! {
        const _: () = {
            #raw_fn
        };

        #item_fn

        #extern_fn
    };
    if !out_fns.is_empty() {
        out.extend(out_fns.iter().map(|out_fn| {out_fn.to_token_stream()}));
    };
    Ok(out)
}

// MARK: Parse output

fn parse_output(
    ty: &Type,
    ident_prefix: &str,
    out_stmts: &mut Vec<Stmt>,
    out_fns: &mut Vec<ItemFn>,
    // itemfn, externfn, rawfn
) -> Result<(Type, IrType)> {

    match ty {
        // Supported types
        Type::Path(TypePath{ ref path, .. }) => {
            let ident = path.get_ident().ok_or(())?.to_string();
            if let Some(trivial_ty) = is_trivial(ident.as_str()) {

                Ok((
                    ty.clone(),
                    IrType::Trivial(trivial_ty)
                ))
            } else {
                match ident.as_str() {
                    "str" => {
                        out_stmts.push(parse_quote!(
                            let out = (out.as_mut_ptr(), out.len());
                        ));
                        let fn_ident = Ident::new(
                            &format!("{}_str_ptr", ident_prefix),
                            Span::mixed_site()
                        );
                        out_fns.push(parse_quote!(
                            #[no_mangle]
                            extern "C" fn #fn_ident(arg_0: *mut (*mut u8, usize)) -> *mut u8 {
                                unsafe { (*arg_0).0 }
                            }
                        ));
                        let fn_ident = Ident::new(
                            &format!("{}_str_len", ident_prefix),
                            Span::mixed_site()
                        );
                        out_fns.push(parse_quote!(
                            #[no_mangle]
                            extern "C" fn #fn_ident(arg_0: *mut (*mut u8, usize)) -> usize {
                                unsafe { (*arg_0).1 }
                            }
                        ));
                        Ok((
                            parse_quote!((*mut u8, usize)),
                            IrType::Str
                        ))
                    },
                    _ => {
                        out_stmts.push(parse_quote!(
                            let out: Box<#ty> = Box::from(out);
                        ));
                        out_stmts.push(parse_quote!(
                            let out = Box::into_raw(out);
                        ));
                        Ok((
                            parse_quote!(*mut #ty),
                            IrType::Custom(ty.clone())
                        ))
                    }
                }
            }
        },
        Type::Ptr(TypePtr{ ref elem, .. }) => {
            let trivial_ty = Trivial::Pointer(*elem.clone());

            Ok((
                *elem.clone(),
                IrType::Trivial(trivial_ty)
            ))
        },

        Type::Reference(TypeReference{ ref elem, .. }) => {

            // box the value to own it and allow it to be mutably referenced
            out_stmts.push(parse_quote!(
                let mut out: Box<#elem> = Box::from(out.to_owned());
            ));
            let (ty, ir_ty) = parse_output(
                &*elem,
                ident_prefix,
                out_stmts,
                out_fns
            )?;
            let ty = match ty {
                Type::Ptr(_) => {
                    ty
                },
                _ => {
                    out_stmts.push(parse_quote!(
                        let out: Box<#ty> = Box::from(out);
                    ));
                    out_stmts.push(parse_quote!(
                        let out: *mut #ty = Box::into_raw(out);
                    ));
                    parse_quote!(*mut #ty)
                },
            };

            Ok((
                ty,
                IrType::Ref(Box::new(ir_ty))
            ))
        },
        Type::Paren(TypeParen{ ref elem, .. }) => {
            out_stmts.push(parse_quote!(
                let (out) = out;
            ));

            let (ty, ir_ty) = parse_output(
                &*elem,
                ident_prefix,
                out_stmts,
                out_fns
            )?;

            Ok((
                ty,
                IrType::Paren(Box::new(ir_ty))
            ))
        },
        Type::Tuple(TypeTuple{ ref elems, .. }) => {
            let mut out_ty: Vec<Type> = Vec::new();
            let mut out_ir_ty: Vec<IrType> = Vec::new();
            let mut out_expr: ExprTuple = parse_quote!(());

            out_stmts.push(parse_quote!(
                mut let out_tup = out;
            ));

            for (index, ty) in elems.iter().enumerate() {
                out_stmts.push(parse_quote!(
                    let out = out_tup.#index;
                ));

                let (ty, ir_ty) = parse_output(
                    ty,
                    &format!("{}_tup_{}", ident_prefix, index),
                    out_stmts,
                    out_fns
                )?;

                let this_ident = Ident::new(
                    &format!("out_{}", index),
                    Span::mixed_site()
                );
                out_stmts.push(parse_quote!(
                    #this_ident = out as #ty;
                ));

                out_expr.elems.push(parse_quote!(#this_ident));

                out_ty.push(ty);
                out_ir_ty.push(ir_ty);
            };
            out_stmts.push(parse_quote!(
                let out = #out_expr;
            ));

            out_stmts.push(parse_quote!(
                let out = Box::from(out);
            ));
            out_stmts.push(parse_quote!(
                let out = Box::into_raw(out);
            ));

            for (index, this_ty) in out_ty.iter().enumerate() {
                let fn_ident = Ident::new(
                    &format!("{}_{}", ident_prefix, index),
                    Span::mixed_site()
                );
                out_fns.push(parse_quote!(
                    #[no_mangle]
                    extern "C" fn #fn_ident(arg_0: *mut (#(#out_ty),*)) -> #this_ty {
                        unsafe { (*arg_0).#index }
                    }
                ));
            };

            Ok((
                parse_quote!( *mut (#(#out_ty),*) ),
                IrType::Tuple(out_ir_ty)
            ))
        },
        Type::Slice(TypeSlice{ ref elem, .. }) => {
            out_stmts.push(parse_quote!(
                let out = (out.as_mut_ptr(), out.len());
            ));
            Ok((
                parse_quote!((*mut #elem, usize)),
                IrType::Slice(*elem.clone())
            ))
        },

        // Unsupported types
        // Type::Array(type_array) => todo!(),
        // Type::BareFn(type_bare_fn) => todo!(),
        // Type::Group(type_group) => todo!(),
        // Type::ImplTrait(type_impl_trait) => todo!(),
        // Type::Infer(type_infer) => todo!(),
        // Type::Macro(type_macro) => todo!(),
        // Type::Never(type_never) => todo!(),
        // Type::TraitObject(type_trait_object) => todo!(),
        // Type::Verbatim(token_stream) => todo!(),
        _ => {
            emit_error!(ty, "unsupported return type");
            Err(())
        },
    }
}

// MARK: Parse input

fn parse_input(
    ty: &Type,
    in_stmts: &mut Vec<Stmt>,
    in_args: &mut Vec<FnArg>
) -> Result<(Expr, IrType)> {
    let arg_new = |index: usize| {
        Ident::new(
            &format!("arg_{}", index),
            Span::mixed_site()
        )
    };
    let index = in_args.len();
    let ident = arg_new(index);

    match ty {
        // Supported types
        Type::Path(TypePath{ ref path, .. }) => {
            let this_ident = path.get_ident().ok_or(())?.to_string();
            if let Some(ty) = is_trivial(this_ident.as_str()) {
                in_args.push(parse_quote!(#ident: #ty));

                Ok((
                    parse_quote!( #ident ),
                    IrType::Trivial(ty)
                ))
            } else {
                match this_ident.as_str() {
                    "str" => {
                        parse_input(
                            &parse_quote!([u8]),
                            in_stmts,
                            in_args
                        )?;

                        in_stmts.push(parse_quote!(
                            let #ident = unsafe { std::str::from_utf8_unchecked_mut(#ident) };
                        ));

                        Ok((
                            parse_quote!( *#ident ),
                            IrType::Str
                        ))
                    },
                    _ => {
                        parse_input(
                            &parse_quote!(*mut #ty),
                            in_stmts,
                            in_args
                        )?;

                        in_stmts.push(parse_quote!(
                            let #ident = unsafe { Box::from_raw(#ident as #ty) }
                        ));

                        Ok((
                            parse_quote!( #ident ),
                            IrType::Custom(ty.clone())
                        ))
                    }
                }
            }
        },
        Type::Ptr(TypePtr{ ref elem, .. }) => {
            let ty = Trivial::Pointer(*elem.clone());
            in_args.push(parse_quote!(#ident: #ty));

            Ok((
                parse_quote!( #ident ),
                IrType::Trivial(ty)
            ))
        },
        Type::Reference(TypeReference{ ref mutability, ref elem, .. }) => {
            let _ref = if mutability.is_some() {
                quote! { &mut }
            } else {
                quote! { & }
            };

            let (expr, ir_ty) = parse_input(
                &*elem,
                in_stmts,
                in_args
            )?;

            Ok((
                parse_quote!( #_ref #expr ),
                ir_ty
            ))
        },
        Type::Paren(TypeParen{ ref elem, .. }) => {
            let (expr, ir_ty)= parse_input(
                &*elem,
                in_stmts,
                in_args
            )?;

            in_stmts.push(parse_quote!(
                let #ident = ( #expr );
            ));

            Ok((
                parse_quote!( #ident ),
                IrType::Paren(Box::new(ir_ty))
            ))
        },
        Type::Tuple(TypeTuple{ ref elems, .. }) => {
            let mut exprs: Vec<Expr> = Vec::new();
            let mut ir_tys: Vec<IrType> = Vec::new();

            for elem in elems {
                let (expr, ir_ty) = parse_input(
                    &*elem,
                    in_stmts,
                    in_args
                )?;

                exprs.push(expr);
                ir_tys.push(ir_ty);
            };

            in_stmts.push(parse_quote!(
                let #ident = (#(#exprs),*);
            ));

            Ok((
                parse_quote!( #ident ),
                IrType::Tuple(ir_tys)
            ))
        },
        Type::Slice(TypeSlice { ref elem, .. }) => {
            let ptr = {
                if let Type::Path(TypePath{ ref path, ..}) = **elem {
                    let ident = path.get_ident().ok_or(())?.to_string();
                    match ident.as_str() {
                        "u8" => {
                            Trivial::Buffer
                        },
                        _ => {
                            Trivial::Pointer(*elem.clone())
                        }
                    }
                } else {
                    Trivial::Pointer(*elem.clone())
                }
            };
            in_args.push(parse_quote!(#ident: #ptr));


            let len = arg_new(index + 1);
            in_args.push(parse_quote!(#len: usize));
            in_stmts.push(parse_quote!(
                let #ident = unsafe { std::slice::from_raw_parts_mut(#ident, #len) };
            ));

            Ok((
                parse_quote!( #ident ),
                IrType::Slice(*elem.clone())
            ))
        },

        // Unsupported types
        // Type::Array(type_array) => todo!(),
        // Type::BareFn(type_bare_fn) => todo!(),
        // Type::Group(type_group) => todo!(),
        // Type::ImplTrait(type_impl_trait) => todo!(),
        // Type::Infer(type_infer) => todo!(),
        // Type::Macro(type_macro) => todo!(),
        // Type::Never(type_never) => todo!(),
        // Type::TraitObject(type_trait_object) => todo!(),
        // Type::Verbatim(token_stream) => todo!(),
        _ => {
            emit_error!(ty, "unsupported return type");
            Err(())
        },
    }
}

// MARK: Utilities

fn is_trivial(ident: &str) -> Option<Trivial> {
    match ident {
        "bool"  => Some(Trivial::Bool),
        "u8"    => Some(Trivial::U8),
        "u16"   => Some(Trivial::U16),
        "u32"   => Some(Trivial::U32),
        "u64"   => Some(Trivial::U64),
        "i8"    => Some(Trivial::I8),
        "i16"   => Some(Trivial::I16),
        "i32"   => Some(Trivial::I32),
        "i64"   => Some(Trivial::I64),
        "usize" => Some(Trivial::Usize),
        "isize" => Some(Trivial::Isize),
        "f32"   => Some(Trivial::F32),
        "f64"   => Some(Trivial::F64),
        _ => None
    }
}

impl ToTokens for Trivial {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            Trivial::Void    => quote! { () },
            Trivial::Bool    => quote! { bool },
            Trivial::U8      => quote! { u8 },
            Trivial::U16     => quote! { u16 },
            Trivial::U32     => quote! { u32 },
            Trivial::U64     => quote! { u64 },
            Trivial::I8      => quote! { i8 },
            Trivial::I16     => quote! { i16 },
            Trivial::I32     => quote! { i32 },
            Trivial::I64     => quote! { i64 },
            Trivial::Usize   => quote! { usize },
            Trivial::Isize   => quote! { isize },
            Trivial::F32     => quote! { f32 },
            Trivial::F64     => quote! { f64 },

            // pointers are mut by default since this is the default semantic for C ABIs
            // this also allows coercion of raw pointers into mutable or immutable references, unlike *const which only allows immutable references
            Trivial::Pointer(ty) => quote! { *mut #ty },
            Trivial::Buffer  => quote! { *mut u8 },
        };

        tokens.extend(iter);
    }
}

impl ToTokens for IrType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            IrType::Trivial(trivial_ty) => quote! {#trivial_ty},
            IrType::Ref(ir_ty) => quote! {*mut #ir_ty},
            IrType::Paren(ir_ty) => quote! {(#ir_ty)},
            IrType::Tuple(ir_tys) => quote! {(#(#ir_tys),*)},
            IrType::Slice(ty) => quote! {[#ty]},
            IrType::Str => quote! {&str},
            IrType::Custom(ty) => quote! {#ty},
        };

        tokens.extend(iter);
    }
}

#[derive(Clone, Debug)]
pub struct ExternFn(ItemFn);
impl Default for ExternFn {
    fn default() -> Self {
        let item_fn: ItemFn = parse_quote!(
            #[no_mangle]
            extern "C" fn __() {}
        );

        ExternFn(item_fn)
    }
}

impl ExternFn {
    pub fn with_ident(mut self, source_fn_ident: &Ident) -> Self {
        self.0.sig.ident = Ident::new(
            &format!("_{}", source_fn_ident.to_string().as_str()),
            Span::mixed_site()
        );
        self
    }

    pub fn with_inputs(&mut self, in_args: Vec<FnArg>, mut in_stmts: Vec<Stmt>) {
        self.0.sig.inputs = Punctuated::from_iter(in_args);
        in_stmts.append(&mut self.0.block.stmts);
        self.0.block.stmts = in_stmts;
    }

    pub fn with_call(&mut self, source_fn_ident: &Ident, exprs: Vec<Expr>) {
        let mut expr_call: ExprCall = parse_quote!(#source_fn_ident());
        expr_call.args = Punctuated::from_iter(exprs);

        let out: Stmt = match self.0.sig.output {
            ReturnType::Default => parse_quote!(#expr_call;),
            ReturnType::Type(..) => parse_quote!(let out = #expr_call;),
        };
        self.0.block.stmts.insert(0, out);
    }

    pub fn with_output(&mut self, out_ty: ReturnType, out_stmts: &mut Vec<Stmt>) {
        self.0.sig.output = out_ty;
        self.0.block.stmts.append(out_stmts);
        self.0.block.stmts.push(Stmt::Expr(parse_quote!(out), None));
    }
}

impl ToTokens for ExternFn {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item_fn = &self.0;
        let iter = quote! {#item_fn};

        tokens.extend(iter);
    }
}