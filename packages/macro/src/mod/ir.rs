use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr, Ident, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Trivial {
    #[default]
    Void,
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Usize,
    Isize,
    F32,
    F64,

    // pointer types
    Pointer(&'static str),
    Buffer,
    // Function, // TODO: unimpeleted types that are supported by the ffi

    // Struct(Box<[Trivial]>),
}

// types that directly appear in the original function
#[derive(Clone, Debug)]
pub enum IrType {
    Trivial(Trivial),
    Ref(Box<IrType>),
    Paren(Box<IrType>),
    Tuple(Vec<IrType>),
    Slice(&'static str), // [pointer, usize]
    Str, // [pointer, usize]
    // String(SliceType),
    // Vec(),
    Custom(&'static str), // [pointer]
}

impl Default for IrType {
    fn default() -> Self {
        IrType::Trivial(Trivial::default())
    }
}

#[derive(Clone, Debug)]
pub enum RawType {
    Trivial(Trivial),
    Ref(&'static RawType),
    Paren(&'static RawType),
    Tuple(&'static [RawType]),
    Slice(Type), // [pointer, usize]
    Str, // [pointer, usize]
    Custom(Type), // [pointer]
}

impl Default for RawType {
    fn default() -> Self {
        RawType::Trivial(Trivial::default())
    }
}

#[derive(Debug, Default)]
pub struct RawFn {
    pub ident: &'static str,
    pub raw_inputs: &'static [RawType],
    pub raw_output: RawType,
    pub non_blocking: bool,
    pub _internal: bool,
    pub _constructor: bool,
}

#[derive(Clone, Debug)]
pub struct RawFnBuilder {
    pub ident: Ident,
    pub raw_inputs:Vec<IrType>,
    pub raw_output: IrType,
    pub non_blocking: bool,
    pub _internal: bool,
    pub _constructor: bool,
}

impl Default for RawFnBuilder {
    fn default() -> Self {
        RawFnBuilder{
            ident: Ident::new("__", Span::mixed_site()),
            raw_inputs: Vec::new(),
            raw_output: IrType::default(),
            non_blocking: false,
            _internal: false,
            _constructor: false,
        }
    }
}

impl RawFnBuilder {
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
            Trivial::Pointer(ty) => parse_quote!(deno_bindgen2::Trivial::Pointer(parse_quote!(#ty))),
            Trivial::Buffer => parse_quote!(deno_bindgen2::Trivial::Buffer),
        }
    }
}

impl IrType {
    fn to_ident(&self) -> syn::Expr {
        match &self {
            IrType::Trivial(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::RawType::Trivial(#expr))
            },
            IrType::Ref(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::RawType::Ref(&#expr))
            },
            IrType::Paren(ty) => {
                let expr = ty.to_ident();
                parse_quote!(deno_bindgen2::RawType::Paren(&#expr))
            },
            IrType::Tuple(tys) => {
                let exprs: Vec<Expr> = tys.iter().map(|ty| {
                    ty.to_ident()
                }).collect();
                parse_quote!(deno_bindgen2::RawType::Tuple(&[#(#exprs),*]))
            },
            IrType::Slice(ty) => {
                parse_quote!(deno_bindgen2::RawType::Slice(#ty))
            },
            IrType::Str => {
                parse_quote!(deno_bindgen2::RawType::Str)
            },
            IrType::Custom(ty) => {
                parse_quote!(deno_bindgen2::RawType::Custom(#ty))
            },
        }
    }
}


impl ToTokens for RawFnBuilder {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let ident = &self.ident;
    let raw_inputs = &self.raw_inputs.iter()
        .map(|ty| {
            ty.to_ident()
        })
        .collect::<Vec<_>>() ;
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
            };
        };

        tokens.extend(iter);
    }
}