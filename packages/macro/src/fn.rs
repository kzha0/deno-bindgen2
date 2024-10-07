use proc_macro2::{
    Ident,
    Span,
    TokenStream,
};
use proc_macro_error::{
    emit_error,
    emit_warning,
};
use quote::{
    format_ident,
    quote,
    ToTokens,
};
use syn::{
    parse_quote, punctuated::Punctuated, Expr, ExprCall, FnArg, Index, ItemFn, Member, PatType, ReturnType, Signature, Stmt, Type, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple
};

pub use crate::*;

// MARK: ENTRYPOINT PARSER

#[derive(Default)]
pub struct MacroArgsFn {
    pub non_blocking: bool,
    pub _internal:    bool,
    pub _constructor: bool,
}

pub fn parse_fn(item_fn: &ItemFn, fn_args: MacroArgsFn) -> Result<(TokenStream, RawFnBuilder)> {
    //-------------------------------- CHECKS ------------------------------/

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

    //-------------------------------- PARSER ------------------------------/

    let mut extern_fn = ExternFn::default().with_ident(&item_fn.sig.ident);
    let mut out_fns: Vec<TokenStream> = Vec::new();

    let raw_fn = RawFnBuilder::default()
        .with_ident(&extern_fn.0.sig.ident)
        .with_output({
            match &item_fn.sig.output {
                ReturnType::Default => IrType::default(),
                ReturnType::Type(_, ty) => {
                    let (
                        out_ty,
                        mut out_stmts,
                        ir_ty,
                        mut this_out_fns
                    ) = parse_output(
                        &*ty,
                        &extern_fn.0.sig.ident.to_string().as_str()
                    )?;

                    out_fns.append(&mut this_out_fns);
                    extern_fn.with_output(parse_quote!(-> #out_ty), &mut out_stmts);
                    ir_ty
                },
            }
        })
        .with_inputs({

            let mut in_args: Vec<FnArg> = Vec::new();
            let mut in_stmts: Vec<Stmt> = Vec::new();

            let mut ir_tys: Vec<IrType> = Vec::new();
            let mut in_exprs: Vec<Expr> = Vec::new();

            for fn_arg in &item_fn.sig.inputs {
                match fn_arg {
                    FnArg::Receiver(receiver) => {
                        emit_error!(receiver, "unsupported self type");
                        return Err(());
                    },
                    FnArg::Typed(PatType { ty, .. }) => {
                        let (
                            this_in_args,
                            mut this_in_stmts,
                            this_ir_ty,
                            this_in_expr
                        ) = parse_input(&*ty, in_args.len())?;

                        for in_arg in this_in_args {
                            in_args.push(FnArg::Typed(in_arg));
                        };
                        in_stmts.append(&mut this_in_stmts);

                        ir_tys.push(this_ir_ty);
                        in_exprs.push(this_in_expr);
                    },
                }
            }

            extern_fn.with_call(&item_fn.sig.ident, in_exprs);
            extern_fn.with_inputs(in_args, in_stmts);
            ir_tys
        })
        .with_fn_args(fn_args);

    let out = quote! {
        #item_fn
        #extern_fn
        #(#out_fns)*
    };
    Ok((out, raw_fn))
}

// MARK: PARSE OUTPUT

fn parse_output(
    ty: &Type,
    ident_prefix: &str
) -> Result<(Type, Vec<Stmt>, IrType, Vec<TokenStream>)> {

    let out_ty: Type;
    let mut out_stmts: Vec<Stmt> = Vec::new();

    let ir_ty: IrType;
    let mut out_fns: Vec<TokenStream> = Vec::new();

    match ty {
        // Supported types
        Type::Path(TypePath { ref path, .. }) => {
            let ident = path.get_ident().ok_or(())?.to_string();
            if let Some(trivial_ty) = is_trivial(&ident) {

                out_ty = ty.clone();
                ir_ty = IrType::Trivial(trivial_ty);

            } else {
                match ident.as_str() {
                    "String" => {
                        let (
                            this_out_ty,
                            _,
                            _,
                            mut this_out_fns
                        ) = parse_output(&parse_quote!(&[u8]), ident_prefix)?;

                        out_stmts.append(&mut parse_quote!(
                            let out = out.into_boxed_str();
                            let out = Box::leak(out);
                            let out = (out.as_mut_ptr(), out.len());
                            let out: Box<(*mut u8, usize)> = Box::from(out);
                            let out = Box::into_raw(out);
                        ));

                        out_ty = this_out_ty;
                        ir_ty = IrType::Str;
                        out_fns.append(&mut this_out_fns);
                    },
                    _ => {
                        out_stmts.append(&mut parse_quote!(
                            let out = Box::from(out);
                            let out = Box::into_raw(out);
                        ));

                        let fn_ident = format_ident!("{}_dealloc", ident_prefix);
                        out_fns.push(quote! {
                            #[no_mangle]
                            extern "C" fn #fn_ident(arg_0: *mut #ty) {
                                unsafe { drop(Box::from_raw(arg_0)) };
                            }
                        });

                        out_ty = parse_quote!(*mut #ty);
                        ir_ty = IrType::Custom(Box::leak(
                            ty.to_token_stream().to_string().into_boxed_str(),
                        ));
                    },
                }
            }
        },
        Type::Ptr(TypePtr { ref elem, .. }) => {
            out_ty = ty.clone();
            ir_ty = IrType::Trivial(Trivial::Pointer(Box::leak(
                elem.to_token_stream().to_string().into_boxed_str(),
            )));
        },
        Type::Reference(TypeReference{ ref elem, .. }) => {
            match **elem {
                Type::Path(TypePath{ ref path, .. }) => {
                    if &path.get_ident().ok_or(())?.to_string() == "str" {

                        let (
                            this_out_ty,
                            _,
                            _,
                            mut this_out_fns
                        ) = parse_output(&parse_quote!(&[u8]), ident_prefix)?;

                        // box the value to own it and allow it to be mutably referenced
                        out_stmts.append(&mut parse_quote!(
                            let out: Box<str> = Box::from(out.to_owned());
                            let out = out.to_string().into_boxed_str();
                            let out = Box::leak(out);
                            let out = (out.as_mut_ptr(), out.len());
                            let out: Box<(*mut u8, usize)> = Box::from(out);
                            let out = Box::into_raw(out);
                        ));

                        out_ty = this_out_ty;
                        ir_ty = IrType::Str;
                        out_fns.append(&mut this_out_fns);
                    } else {
                        out_stmts.append(&mut parse_quote!(
                            let out = Box::from(out);
                            let out = Box::into_raw(out);
                        ));
                        out_ty = parse_quote!(*mut #elem);
                        ir_ty = IrType::Trivial(Trivial::Pointer(Box::leak(
                            elem.to_token_stream().to_string().into_boxed_str(),
                        )));
                    }
                },
                Type::Reference(_)
                | Type::Ptr(_) => {
                    let (
                        this_out_ty,
                        mut this_out_stmts,
                        this_ir_ty,
                        mut this_out_fns
                    ) = parse_output(&*elem, ident_prefix)?;

                    out_ty = this_out_ty;
                    out_stmts.append(&mut this_out_stmts);
                    ir_ty = this_ir_ty;
                    out_fns.append(&mut this_out_fns);
                },
                Type::Slice(TypeSlice{ ref elem, .. }) => {

                    // TODO: create iterator for non-trivial types
                    // Deno FFI API does not support interpreting arrays of pointers

                    let ptr = {
                        if let Type::Path(TypePath{ ref path, .. }) = **elem {
                            let this_ident = path.get_ident().ok_or(())?.to_string();
                            if let Some(trivial_ty) = is_numeric(&this_ident) {
                                Ok(Trivial::Buffer(Box::new(trivial_ty)))

                            } else {
                                Err(())
                            }
                        } else {
                            Err(())
                        }
                    }.or_else(|_| {
                        emit_error!(
                            elem, "unsupported slice type: {}", elem.to_token_stream().to_string();
                            note = "deno_bindgen2 currently does not support non-trivial slices";
                        );
                        Err(())
                    })?;

                    out_stmts.append(&mut parse_quote!(
                        let out: Box<[#elem]> = Box::from(out.to_owned());
                        let out = Box::leak(out);
                        let out = (out.as_mut_ptr(), out.len());
                        let out: Box<(*mut #elem, usize)> = Box::from(out);
                        let out = Box::into_raw(out);
                    ));

                    // TODO: use the tuple parser to programatically generate these functions

                    let fn_ident = format_ident!("{}_ptr", ident_prefix);
                    out_fns.push(quote! {
                        #[no_mangle]
                        extern "C" fn #fn_ident(arg_0: *mut (*mut #elem, usize)) -> *mut #elem {
                            unsafe { (*arg_0).0 }
                        }
                    });

                    let fn_ident = format_ident!("{}_len", ident_prefix);
                    out_fns.push(quote! {
                        #[no_mangle]
                        extern "C" fn #fn_ident(arg_0: *mut (*mut #elem, usize)) -> usize {
                            unsafe { (*arg_0).1 }
                        }
                    });

                    let fn_ident = format_ident!("{}_dealloc", ident_prefix);
                    out_fns.push(quote! {
                        #[no_mangle]
                        extern "C" fn #fn_ident(arg_0: *mut (*mut #elem, usize)) {
                            unsafe { drop(std::slice::from_raw_parts_mut((*arg_0).0, (*arg_0).1)) };
                            unsafe { drop(Box::from_raw(arg_0)) };
                        }
                    });

                    out_ty = parse_quote!(*mut (*mut #elem, usize));
                    ir_ty = IrType::Slice(Box::new(IrType::Trivial(ptr)));
                },
                _ => {
                    out_stmts.append(&mut parse_quote!(
                        let out = Box::from(out);
                        let out = Box::into_raw(out);
                    ));
                    out_ty = parse_quote!(*mut #elem);
                    ir_ty = IrType::Trivial(Trivial::Pointer(Box::leak(
                        elem.to_token_stream().to_string().into_boxed_str(),
                    )));
                },
            };
        },
        Type::Paren(TypeParen { ref elem, .. }) => {
            out_stmts.push(parse_quote!(
                let (out) = out;
            ));

            let (
                this_out_ty,
                mut this_out_stmts,
                this_ir_ty,
                mut this_out_fns
            ) = parse_output(&*elem, ident_prefix)?;

            out_stmts.append(&mut this_out_stmts);
            out_ty = this_out_ty;
            ir_ty = IrType::Paren(Box::new(this_ir_ty));
            out_fns.append(&mut this_out_fns);
        },
        Type::Tuple(TypeTuple { ref elems, .. }) => {

            let mut tup_out_tys: Vec<Type> = Vec::new();
            let mut tup_ir_tys: Vec<IrType> = Vec::new();
            let mut tup_exprs: Vec<Expr> = Vec::new();
            let mut tup_out_fns: Vec<Option<Vec<TokenStream>>> = Vec::new();

            out_stmts.push(parse_quote!(
                let mut out_tup = out;
            ));

            for (index, elem) in elems.iter().enumerate() {
                let member = Member::Unnamed(Index::from(index));
                out_stmts.push(parse_quote!(
                    let out = out_tup.#member;
                ));

                let (
                    this_out_ty,
                    mut this_out_stmts,
                    this_ir_ty,
                    this_out_fns
                ) = parse_output(&*elem, &format!("{}_{}", ident_prefix, index))?;

                out_stmts.append(&mut this_out_stmts);

                let this_ident = format_ident!("out_tup_{}", index);
                out_stmts.push(parse_quote!(
                    let #this_ident = out as #this_out_ty;
                ));

                tup_out_tys.push(this_out_ty);
                tup_ir_tys.push(this_ir_ty);

                tup_exprs.push(parse_quote!(#this_ident));
                if this_out_fns.len() > 0 {
                    tup_out_fns.push(Some(this_out_fns));
                } else {
                    tup_out_fns.push(None);
                };
            }

            out_stmts.append(&mut parse_quote!(
                let out = (#(#tup_exprs),*);
                let out = Box::from(out);
                let out = Box::into_raw(out);
            ));

            for (index, this_out_ty) in tup_out_tys.iter().enumerate() {
                let fn_ident = format_ident!("{}_{}", ident_prefix, index);
                let member = Member::Unnamed(Index::from(index));

                out_fns.push(quote! {
                    #[no_mangle]
                    extern "C" fn #fn_ident(arg_0: *mut (#(#tup_out_tys),*)) -> #this_out_ty {
                        unsafe { (*arg_0).#member }
                    }
                });

                if let Some(out_fn) = &tup_out_fns[index] {
                    out_fns.extend(out_fn.clone());
                }
            }

            let fn_ident = format_ident!("{}_dealloc", ident_prefix);
            let tup_members: Vec<Expr> = tup_out_tys.iter().enumerate().map(|(index, _)| {
                let member = Member::Unnamed(Index::from(index));
                parse_quote!((*arg_0).#member)
            }).collect();
            out_fns.push(quote! {
                #[no_mangle]
                extern "C" fn #fn_ident(arg_0: *mut (#(#tup_out_tys),*)) {
                    unsafe { drop((#(#tup_members),*)) };
                    unsafe { drop(Box::from_raw(arg_0)) };
                }
            });

            out_ty = parse_quote!( *mut (#(#tup_out_tys),*) );
            ir_ty = IrType::Tuple(tup_ir_tys);
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
            return Err(())
        },
    };

    Ok((out_ty, out_stmts, ir_ty, out_fns))
}

// MARK: PARSE INPUTS

fn parse_input(
    ty: &Type,
    index: usize
) -> Result<(Vec<PatType>, Vec<Stmt>, IrType, Expr)> {
    let ident = format_ident!("arg_{index}");

    let mut in_args: Vec<PatType> = Vec::new();
    let mut in_stmts: Vec<Stmt> = Vec::new();
    let in_expr: Expr;
    let ir_ty: IrType;

    match ty {
        // Supported types
        Type::Path(TypePath { ref path, .. }) => {
            let this_ident = path.get_ident().ok_or(())?.to_string();
            if let Some(trivial_ty) = is_trivial(&this_ident) {

                in_args.push(parse_quote!(#ident: #trivial_ty));
                ir_ty = IrType::Trivial(trivial_ty);
                in_expr = parse_quote!(#ident);

            } else {
                match this_ident.as_str() {
                    "String" => {
                        let (
                            mut this_in_args,
                            mut this_in_stmts,
                            ..
                        ) = parse_input(&parse_quote!(&[u8]), index)?;

                        in_args.append(&mut this_in_args);
                        in_stmts.append(&mut this_in_stmts);
                        in_stmts.push(parse_quote!(
                            let #ident = unsafe { String::from_utf8_unchecked(#ident.to_vec()) };
                        ));

                        ir_ty = IrType::Str;
                        in_expr = parse_quote!( #ident );
                    },
                    _ => {
                        let (
                            mut this_in_args,
                            mut this_in_stmts,
                            ..
                        ) = parse_input(&parse_quote!(*mut #ty), index)?;

                        in_args.append(&mut this_in_args);
                        in_stmts.append(&mut this_in_stmts);
                        in_stmts.push(parse_quote!(
                            let #ident = unsafe { *(Box::from_raw(#ident)) };
                        ));

                        ir_ty = IrType::Custom(Box::leak(
                            ty.to_token_stream().to_string().into_boxed_str(),
                        ));
                        in_expr = parse_quote!( #ident );
                    },
                }
            }
        },
        Type::Ptr(TypePtr{ ref elem, .. }) => {
            let trivial_ty = Trivial::Pointer(Box::leak(
                elem.to_token_stream().to_string().into_boxed_str(),
            ));

            in_args.push(parse_quote!(#ident: #ty));

            ir_ty = IrType::Trivial(trivial_ty);
            in_expr = parse_quote!(#ident);
        },
        Type::Reference(TypeReference { ref elem, ..}) => {
            match **elem {
                Type::Path(TypePath{ ref path, .. }) => {
                    if &path.get_ident().ok_or(())?.to_string() == "str" {
                        let (
                            mut this_in_args,
                            mut this_in_stmts,
                            ..
                        ) = parse_input(&parse_quote!(&[u8]), index)?;

                        in_args.append(&mut this_in_args);

                        in_stmts.append(&mut this_in_stmts);
                        in_stmts.push(parse_quote!(
                            let #ident = unsafe { std::str::from_utf8_unchecked_mut(#ident) };
                        ));

                        ir_ty = IrType::Str;
                        in_expr = parse_quote!(#ident);
                    } else {
                        in_args.push(parse_quote!(#ident: *mut #elem));

                        in_stmts.push(parse_quote!(
                            let mut #ident = unsafe { *#ident };
                        ));

                        ir_ty = IrType::Trivial(Trivial::Pointer(Box::leak(
                            elem.to_token_stream().to_string().into_boxed_str(),
                        )));
                        in_expr = parse_quote!(&mut #ident);
                    };
                },
                Type::Reference(_)
                | Type::Ptr(_) => {
                    let (
                        mut this_in_args,
                        mut this_in_stmts,
                        this_ir_ty,
                        this_in_expr
                    ) = parse_input(&*elem, index)?;

                    in_args.append(&mut this_in_args);
                    in_stmts.append(&mut this_in_stmts);
                    ir_ty = this_ir_ty;
                    in_expr = this_in_expr;
                },
                Type::Slice(TypeSlice { ref elem, .. }) => {

                    // TODO: create iterator for non-trivial types
                    // Deno FFI API does not support interpreting arrays of pointers

                    let ptr = {
                        if let Type::Path(TypePath { ref path, .. }) = **elem {
                            let this_ident = path.get_ident().ok_or(())?.to_string();
                            if let Some(trivial_ty) = is_numeric(&this_ident) {
                                Ok(Trivial::Buffer(Box::new(trivial_ty)))

                            } else {
                                Err(())
                            }
                        } else {
                            Err(())
                        }
                    }.or_else(|_| {
                        emit_error!(
                            elem, "unsupported slice type: {}", elem.to_token_stream().to_string();
                            note = "deno_bindgen2 currently does not support non-trivial slices";
                        );
                        Err(())
                    })?;

                    in_args.push(parse_quote!(#ident: #ptr));

                    let len = format_ident!("arg_{}", index + 1);
                    in_args.push(parse_quote!(#len: usize));

                    in_stmts.push(parse_quote!(
                        let mut #ident = unsafe { std::slice::from_raw_parts_mut(#ident, #len) };
                    ));

                    ir_ty = IrType::Slice(Box::new(IrType::Trivial(ptr)));
                    in_expr = parse_quote!( #ident );
                },
                _ => {
                    in_args.push(parse_quote!(#ident: *mut #elem));

                    in_stmts.push(parse_quote!(
                        let mut #ident = unsafe { *#ident };
                    ));

                    ir_ty = IrType::Trivial(Trivial::Pointer(Box::leak(
                        elem.to_token_stream().to_string().into_boxed_str(),
                    )));
                    in_expr = parse_quote!(&mut #ident);
                },
            };
        },
        Type::Paren(TypeParen { ref elem, .. }) => {
            let (
                mut this_in_args,
                mut this_in_stmts,
                this_ir_ty,
                this_in_expr
            ) = parse_input(&*elem, index)?;

            in_args.append(&mut this_in_args);
            in_stmts.append(&mut this_in_stmts);
            in_stmts.push(parse_quote!(
                let #ident = (#this_in_expr);
            ));

            ir_ty = IrType::Paren(Box::new(this_ir_ty));
            in_expr = parse_quote!(#ident);
        },
        Type::Tuple(TypeTuple { ref elems, .. }) => {
            let mut tup_exprs: Vec<Expr> = Vec::new();
            let mut tup_ir_tys: Vec<IrType> = Vec::new();

            for elem in elems {
                let (
                    mut this_in_args,
                    mut this_in_stmts,
                    this_ir_ty,
                    this_in_expr
                ) = parse_input(&*elem, index + in_args.len())?;

                in_args.append(&mut this_in_args);
                in_stmts.append(&mut this_in_stmts);

                tup_ir_tys.push(this_ir_ty);
                tup_exprs.push(this_in_expr);
            }

            in_stmts.push(parse_quote!(
                let #ident = (#(#tup_exprs),*);
            ));

            ir_ty = IrType::Tuple(tup_ir_tys);
            in_expr = parse_quote!(#ident);
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
            emit_error!(ty, "unsupported input type");
            return Err(())
        },
    };

    Ok((in_args, in_stmts, ir_ty, in_expr))
}

// MARK: CONSTRUCTORS

#[derive(Clone, Debug)]
pub struct RawFnBuilder {
    pub ident:        Ident,
    pub raw_inputs:   Vec<IrType>,
    pub raw_output:   IrType,
    pub non_blocking: bool,
    pub _internal:    bool,
    pub _constructor: bool,
}

impl Default for RawFnBuilder {
    fn default() -> Self {
        RawFnBuilder {
            ident:        Ident::new("__", Span::mixed_site()),
            raw_inputs:   Vec::new(),
            raw_output:   IrType::default(),
            non_blocking: false,
            _internal:    false,
            _constructor: false,
        }
    }
}

impl RawFnBuilder {
    pub fn with_fn_args(
        mut self,
        MacroArgsFn {
            non_blocking,
            _internal,
            _constructor,
        }: MacroArgsFn,
    ) -> Self {
        self.non_blocking = non_blocking;
        self._internal = _internal;
        self._constructor = _constructor;
        self
    }
    pub fn with_ident(mut self, ident: &Ident) -> Self {
        self.ident = ident.clone();
        self
    }

    pub fn with_inputs(mut self, tys: Vec<IrType>) -> Self {
        self.raw_inputs = tys;
        self
    }

    pub fn with_output(mut self, ty: IrType) -> Self {
        self.raw_output = ty;
        self
    }
}

impl ToTokens for RawFnBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;
        let raw_inputs = &self
            .raw_inputs
            .iter()
            .map(|ty| ty.to_ident())
            .collect::<Vec<_>>();
        let raw_output = &self.raw_output.to_ident();
        let non_blocking = &self.non_blocking;
        let _internal = &self._internal;
        let _constructor = &self._constructor;

        let iter = quote! {
            deno_bindgen2::RawFn{
                ident: stringify!(#ident),
                raw_inputs: &[#(#raw_inputs),*],
                raw_output: #raw_output,
                non_blocking: #non_blocking,
                _internal: #_internal,
                _constructor: #_constructor,
            }
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
        self.0.sig.ident = format_ident!("_{}", source_fn_ident);
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

// MARK: UTILITY TYPES

#[derive(Clone, Debug, PartialEq)]
pub enum IrType {
    Trivial(Trivial),
    Paren(Box<IrType>),
    Tuple(Vec<IrType>),
    Slice(Box<IrType>), // [pointer, usize]
    Str,                 // [pointer, usize]
    // Vec(),
    Custom(&'static str), // [pointer]
}

impl Default for IrType {
    fn default() -> Self {
        IrType::Trivial(Trivial::default())
    }
}

impl ToTokens for IrType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            IrType::Trivial(trivial_ty) => quote! {#trivial_ty},
            IrType::Paren(ir_ty) => quote! {(#ir_ty)},
            IrType::Tuple(ir_tys) => quote! {(#(#ir_tys),*)},
            IrType::Slice(ty) => quote! {[#ty]},
            IrType::Str => quote! {&str},
            IrType::Custom(ty) => {
                let ty = format_ident!("{}", ty);
                quote! {#ty}
            },
        };

        tokens.extend(iter);
    }
}

impl IrType {
    fn to_ident(&self) -> syn::Expr {
        match &self {
            IrType::Trivial(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::RawType::Trivial(#expr))
            },
            IrType::Paren(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::RawType::Paren(&#expr))
            },
            IrType::Tuple(tys) => {
                let exprs: Vec<Expr> = tys.iter().map(|ty| ty.to_ident()).collect();
                parse_quote!(deno_bindgen2::RawType::Tuple(&[#(#exprs),*]))
            },
            IrType::Slice(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::RawType::Slice(&#expr))
            },
            IrType::Str => parse_quote!(deno_bindgen2::RawType::Str),
            IrType::Custom(ty) => parse_quote!(deno_bindgen2::RawType::Custom(#ty)),
        }
    }
}

fn is_numeric(ident: &str) -> Option<Trivial> {
    match ident {
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
        _ => None,
    }
}

fn is_trivial(ident: &str) -> Option<Trivial> {
    match ident {
        "bool" => Some(Trivial::Bool),
        _ => is_numeric(ident),
    }
}

impl ToTokens for Trivial {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            Trivial::Void  => quote! { () },
            Trivial::Bool  => quote! { bool },
            Trivial::U8    => quote! { u8 },
            Trivial::U16   => quote! { u16 },
            Trivial::U32   => quote! { u32 },
            Trivial::U64   => quote! { u64 },
            Trivial::I8    => quote! { i8 },
            Trivial::I16   => quote! { i16 },
            Trivial::I32   => quote! { i32 },
            Trivial::I64   => quote! { i64 },
            Trivial::Usize => quote! { usize },
            Trivial::Isize => quote! { isize },
            Trivial::F32   => quote! { f32 },
            Trivial::F64   => quote! { f64 },

            // pointers are mut by default since this is the default semantic for C ABIs
            // this also allows coercion of raw pointers into mutable or immutable references, unlike *const which only allows immutable references
            Trivial::Pointer(ty) => quote! { *mut #ty },
            Trivial::Buffer(ty) => quote!{ *mut #ty }
        };

        tokens.extend(iter);
    }
}

impl Trivial {
    fn to_ident(&self) -> syn::Expr {
        match &self {
            Trivial::Void  => parse_quote!(deno_bindgen2::Trivial::Void),
            Trivial::Bool  => parse_quote!(deno_bindgen2::Trivial::Bool),
            Trivial::U8    => parse_quote!(deno_bindgen2::Trivial::U8),
            Trivial::U16   => parse_quote!(deno_bindgen2::Trivial::U16),
            Trivial::U32   => parse_quote!(deno_bindgen2::Trivial::U32),
            Trivial::U64   => parse_quote!(deno_bindgen2::Trivial::U64),
            Trivial::I8    => parse_quote!(deno_bindgen2::Trivial::I8),
            Trivial::I16   => parse_quote!(deno_bindgen2::Trivial::I16),
            Trivial::I32   => parse_quote!(deno_bindgen2::Trivial::I32),
            Trivial::I64   => parse_quote!(deno_bindgen2::Trivial::I64),
            Trivial::Usize => parse_quote!(deno_bindgen2::Trivial::Usize),
            Trivial::Isize => parse_quote!(deno_bindgen2::Trivial::Isize),
            Trivial::F32   => parse_quote!(deno_bindgen2::Trivial::F32),
            Trivial::F64   => parse_quote!(deno_bindgen2::Trivial::F64),
            Trivial::Pointer(ty) => {
                parse_quote!(deno_bindgen2::Trivial::Pointer(stringify!(#ty)))
            },
            Trivial::Buffer(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::Trivial::Buffer(&#expr))
            },
        }
    }
}