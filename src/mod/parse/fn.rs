use proc_macro2::{TokenStream, Ident, Span};
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, ToTokens};
use syn::{parse_quote, punctuated::Punctuated, token::Comma, Expr, FnArg, ItemFn, PatType, Stmt, Type, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple, Signature};

pub use crate::r#mod::*;

pub struct FnArgs {
    pub optional:     bool,
    pub non_blocking: bool,
    pub _internal:    bool,
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

    let mut symbol_inputs: Vec<IrType> = Vec::new(); // arg names to assemble inputs from
    let mut symbol_output: Vec<SymbolSignature> = Vec::new(); // function names to assemble a non-trivial result from

    let mut ffi_inputs: Punctuated<FnArg, Comma> = Punctuated::new();
    let mut ffi_stmts : Vec<Stmt> = Vec::new();
    let mut ffi_call_args: Vec<Expr> = Vec::new();

    let mut counter: usize = 0;
    for fn_input in &item_fn.sig.inputs {
        match fn_input {
            // receivers should never match and should silently coerce to a custom type
            FnArg::Receiver(rcvr) => {
                emit_error!(rcvr, "unsupported self type");
                return item_fn.to_token_stream()
            },
            FnArg::Typed(PatType{ty, ..}) => {
                // should return
                // Ident: name of argument to pass to fn_call_args
                // an optional nested expression
                if let Ok((expr, ir_ty)) = parse_syn_ty(
                    ty,
                    &mut ffi_inputs,
                    &mut ffi_stmts,
                    &mut counter,
                ) {
                    // assign transform statements here
                    ffi_call_args.push(expr);
                    symbol_inputs.push(ir_ty);
                } else {
                    emit_error!(ty, "unsupported type");
                    return item_fn.to_token_stream()
                }
            },
        }
    };

    let ffi_out: Stmt = {
        let ident = item_fn.sig.ident.clone();
        match item_fn.sig.output {
            syn::ReturnType::Default => {
                // unnecessary semicolon at the end. the original function is a unit result and so is the same for this extern function
                parse_quote!(#ident(#(#ffi_call_args),*);)
            },
            syn::ReturnType::Type(_, _) => {
                parse_quote!(let out = #ident(#(#ffi_call_args),*);)
            },
        }
    };
    ffi_stmts.push(ffi_out);

    let ffi_ident = Ident::new(&format!("__{}", item_fn.sig.ident.to_string()), Span::mixed_site());
    let mut ffi: ItemFn = parse_quote!(
        extern "C" fn #ffi_ident(#ffi_inputs) {

        }
    );
    ffi.block.stmts = ffi_stmts;

    // support parse return type

    let mut out = TokenStream::new();
    out.extend(item_fn.to_token_stream());
    out.extend(ffi.to_token_stream());

    out
}

fn parse_syn_ty(
    ty: &Type,
    ffi_inputs: &mut Punctuated<FnArg, Comma>,
    ffi_stmts: &mut Vec<Stmt>,
    counter: &mut usize,
) -> Result<(Expr, IrType)> {
    match ty {
        Type::Path(TypePath{ ref path, .. }) => {

            let ty = path.get_ident().ok_or(())?;
            if let Some(ty) = is_trivial(ty.to_string().as_str()) {
                let ident = ident_new(counter);

                ffi_inputs.push(parse_quote!( #ident: #ty ));
                let param = ParameterType{
                    ty,
                    ident: ident.clone()
                };

                Ok((parse_quote!( #ident ), IrType::Parameter(param)))
            } else { match ty.to_string().as_str() {
                "str" => {
                    let ident = ident_new(counter);
                    let slice = slice_new(&ident, &parse_quote!( u8 ), ffi_inputs, ffi_stmts, counter);

                    ffi_stmts.push(parse_quote!(
                        let #ident = unsafe { std::str::from_utf8_unchecked_mut(#ident) };
                    ));

                    Ok((parse_quote!( *#ident ), IrType::Str(slice)))
                },
                // "String" => {
                //     // strings and vec add complexity to memory management due to having an additional capacity allocation which must not be violated
                //     todo!()
                // },
                _ => {
                    let ident = ident_new(counter);
                    let ptr = pointer_new(&ident, &parse_quote!( #ty ), ffi_inputs);

                    ffi_stmts.push(parse_quote!(
                        let #ident = unsafe { Box::from_raw(#ident as _) };
                    ));

                    let cstm = CustomType{
                        ty: ty.clone(),
                        ptr
                    };
                    Ok((parse_quote!( #ident ), IrType::Custom(cstm)))
                }
            }}
        },
        Type::Ptr(TypePtr{ ref elem, .. }) => {
            let ident = ident_new(counter);
            let ptr = pointer_new(&ident, elem, ffi_inputs);

            Ok((parse_quote!( #ident ), IrType::Parameter(ptr)))
        },
        Type::Reference(TypeReference{ ref mutability, ref elem, .. }) => {
            let _ref = if mutability.is_some() {
                quote! { &mut }
            } else {
                quote! { & }
            };

            let (expr, ir_ty) = parse_syn_ty(
                elem,
                ffi_inputs,
                ffi_stmts,
                counter
            )?;

            let ref_ty = ReferenceType{
                _mut: mutability.is_some(),
                elem: Box::new(ir_ty)
            };

            Ok((parse_quote!( #_ref #expr ), IrType::Reference(ref_ty)))
        },
        Type::Paren(TypeParen{ ref elem, .. }) => {
            let (expr, ir_ty) = parse_syn_ty(
                elem,
                ffi_inputs,
                ffi_stmts,
                counter
            )?;

            let paren_ty = ParenType{
                elem: Box::new(ir_ty)
            };

            Ok((parse_quote!( (#expr) ), IrType::Paren(paren_ty)))
        },
        Type::Tuple(TypeTuple{ ref elems, .. }) => {
            let mut exprs: Vec<Expr> = Vec::new();
            let mut tys: Vec<IrType> = Vec::new();

            for elem in elems {
                let (expr, ir_ty) = parse_syn_ty(
                    elem,
                    ffi_inputs,
                    ffi_stmts,
                    counter
                )?;
                exprs.push(expr);
                tys.push(ir_ty);
            };

            let tup_ty = TupleType{
                elems: tys
            };

            Ok((parse_quote!( (#(#exprs),*) ), IrType::Tuple(tup_ty)))

        },
        Type::Slice(TypeSlice{ ref elem, ..}) => {
            let ident = ident_new(counter);
            let slice = slice_new(&ident, &*elem, ffi_inputs, ffi_stmts, counter);

            Ok((parse_quote!( #ident ), IrType::Slice(slice)))
        },

        // unsupported
        // Type::Array(type_array) => Err(()),
        // Type::BareFn(type_bare_fn) => Err(()),
        // Type::Group(type_group) => Err(()),
        // Type::ImplTrait(type_impl_trait) => Err(()),
        // Type::Infer(type_infer) => Err(()),
        // Type::Macro(type_macro) => Err(()),
        // Type::Never(type_never) => Err(()),
        // Type::TraitObject(type_trait_object) => Err(()),
        // Type::Verbatim(token_stream) => Err(()),
        _ => Err(()),
    }
}

fn is_trivial(ident: &str) -> Option<TrivialType> {
    match ident {
        "bool"  => Some(TrivialType::Bool),
        "u8"    => Some(TrivialType::U8),
        "u16"   => Some(TrivialType::U16),
        "u32"   => Some(TrivialType::U32),
        "u64"   => Some(TrivialType::U64),
        "i8"    => Some(TrivialType::I8),
        "i16"   => Some(TrivialType::I16),
        "i32"   => Some(TrivialType::I32),
        "i64"   => Some(TrivialType::I64),
        "usize" => Some(TrivialType::Usize),
        "isize" => Some(TrivialType::Isize),
        "f32"   => Some(TrivialType::F32),
        "f64"   => Some(TrivialType::F64),
        _ => None
    }
}

impl ToTokens for TrivialType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            TrivialType::Void    => quote! { () },
            TrivialType::Bool    => quote! { bool },
            TrivialType::U8      => quote! { u8 },
            TrivialType::U16     => quote! { u16 },
            TrivialType::U32     => quote! { u32 },
            TrivialType::U64     => quote! { u64 },
            TrivialType::I8      => quote! { i8 },
            TrivialType::I16     => quote! { i16 },
            TrivialType::I32     => quote! { i32 },
            TrivialType::I64     => quote! { i64 },
            TrivialType::Usize   => quote! { usize },
            TrivialType::Isize   => quote! { isize },
            TrivialType::F32     => quote! { f32 },
            TrivialType::F64     => quote! { f64 },

            // pointers are mut by default since this is the default semantic for C ABIs
            // this also allows coercion of raw pointers into mutable or immutable references, unlike *const which only allows immutable references
            TrivialType::Pointer(ident) => quote! { *mut #ident },
            TrivialType::Buffer  => quote! { *mut u8 },
        };

        tokens.extend(iter);
    }
}

impl ToTokens for ParameterType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ParameterType { ty, ident } = self;
        tokens.extend(quote! {#ident: #ty});
    }
}

fn ident_new(counter: &mut usize) -> Ident {
    *counter += 1;
    Ident::new(
        &format!("arg_{}", *counter - 1),
        Span::mixed_site()
    )
}

fn pointer_new(
    ident: &Ident,
    ty: &Type,
    ffi_inputs: &mut Punctuated<FnArg, Comma>
) -> ParameterType {
    ffi_inputs.push(parse_quote!( #ident: *mut #ty ));
    ParameterType{
        ty: TrivialType::Pointer(ty.clone()),
        ident: ident.clone()
    }
}

fn slice_new(
    ident: &Ident,
    ty: &Type,
    ffi_inputs: &mut Punctuated<FnArg, Comma>,
    ffi_stmts: &mut Vec<Stmt>,
    counter: &mut usize
) -> SliceType {

    let ptr = {
        if let Type::Path(TypePath{ ref path, .. }) = *ty {
            if let Some(buf_ident) = path.get_ident() {
                if buf_ident.to_string().as_str() == "u8" {

                    ffi_inputs.push(parse_quote!( #ident: *mut u8 ));

                    ParameterType{
                        ty: TrivialType::Buffer,
                        ident: ident.clone()
                    }

                } else {
                    pointer_new(ident, ty, ffi_inputs)
                }
            } else {
                pointer_new(ident, ty, ffi_inputs)
            }
        } else {
            pointer_new(ident, ty, ffi_inputs)
        }
    };

    let len = ident_new(counter);
    ffi_inputs.push(parse_quote!( #len: usize ));
    ffi_stmts.push(parse_quote!(
        let #ident = unsafe { std::slice::from_raw_parts_mut(#ident, #len) };
    ));

    SliceType{
        ptr,
        len: ParameterType{
            ty: TrivialType::Usize,
            ident: len
        }
    }
}