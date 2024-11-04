use crate::deno::util::*;
use crate::deno::FfiType;
use crate::rust::{self, Type, TypeNumeric};

/* -------------------------------------------------------------------------- */

// MARK: api

#[derive(Clone, Debug)]
pub enum TsType {
    Void,
    Boolean,
    Number,
    BigInt,
    String,
    TypedArray(TypedArray),
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    Rust(RustType),
}

#[derive(Clone, Debug)]
pub enum TypedArray {
    Uint8Array,
    Uint16Array,
    Uint32Array,
    Uint64Array,
    Int8Array,
    Int16Array,
    Int32Array,
    Int64Array,
    Float32Array,
    Float64Array,
}

#[derive(Clone, Debug)]
pub enum RustType {
    Char,
    FnPointer(String), // pointer object containing actual reference value and signature value
    Pointer(TypeReference),
    Reference(TypeReference),
    Box(Box<TsType>),
    Str,
    String,
    Slice(Box<TsType>),
    Vec(Box<TsType>),
    Tuple(Vec<TsType>),
    UserDefined(String),
    Unsupported, // generic deno pointer object
}

#[derive(Clone, Debug)]
pub struct TypeReference {
    mut_: bool,
    elem: Box<TsType>,
}

/* -------------------------------------------------------------------------- */

// MARK: helpers

#[derive(Clone, Debug, Default)]
pub struct RustTypeList {
    pub fn_pointer: bool,
    pub pointer:    bool,
    pub reference:  bool,
    pub box_:       bool,
    pub str:        bool,
    pub string:     bool,
    pub slice:      bool,
    pub vec:        bool,
    pub tuple:      bool,
}


#[derive(Clone, Debug, Default)]
pub struct UserDefined {
    inner: Vec<String>,
}

impl UserDefined {
    fn insert(&mut self, value: String) {
        if self
            .inner
            .iter()
            .find(|ident| {
                if ident.as_str() == value.as_str() {
                    true
                } else {
                    false
                }
            })
            .is_none()
        {
            self.inner.push(value);
        }
    }
    fn iter(&self) -> std::slice::Iter<'_, String> {
        self.inner.iter()
    }
}

/* -------------------------------------------------------------------------- */

// MARK:  convert

#[rustfmt::skip]
impl Type {
    pub fn unwrap(
        &self,
        rust_types: &mut RustTypeList,
        user_defined: &mut UserDefined,
    ) -> (FfiType, TsType) {
        match self {
            Type::Void => (FfiType::Void, TsType::Void),
            Type::Numeric(type_numeric) => {
                match type_numeric {
                    TypeNumeric::U8    => (FfiType::U8,    TsType::Number),
                    TypeNumeric::U16   => (FfiType::U16,   TsType::Number),
                    TypeNumeric::U32   => (FfiType::U32,   TsType::Number),
                    TypeNumeric::I8    => (FfiType::I8,    TsType::Number),
                    TypeNumeric::I16   => (FfiType::I16,   TsType::Number),
                    TypeNumeric::I32   => (FfiType::I32,   TsType::Number),
                    TypeNumeric::F32   => (FfiType::F32,   TsType::Number),
                    TypeNumeric::F64   => (FfiType::F64,   TsType::Number),
                    TypeNumeric::U64   => (FfiType::U64,   TsType::BigInt),
                    TypeNumeric::I64   => (FfiType::I64,   TsType::BigInt),
                    TypeNumeric::Usize => (FfiType::Usize, TsType::BigInt),
                    TypeNumeric::Isize => (FfiType::Isize, TsType::BigInt),
                }
            },
            Type::Bool => (FfiType::Bool, TsType::Boolean),
            Type::Char => (FfiType::U32,  TsType::Rust(RustType::Char)),
            Type::FnPointer(type_bare_fn) =>{
                rust_types.fn_pointer = true;
                (FfiType::FnPointer, TsType::Rust(RustType::FnPointer(format!("{:#?}", type_bare_fn))))
            },
            rest => {
                let rust_type = match rest {
                    Type::Pointer(rust::TypeReference { mut_, elem }) => {
                        rust_types.pointer = true;
                        let elem = Box::new(elem.unwrap(rust_types, user_defined).1);
                        RustType::Pointer(TypeReference {
                            mut_: *mut_,
                            elem,
                        })
                    },
                    Type::Reference(rust::TypeReference { mut_, elem }) => {
                        rust_types.reference = true;
                        let elem = Box::new(elem.unwrap(rust_types, user_defined).1);
                        RustType::Reference(TypeReference {
                            mut_: *mut_,
                            elem,
                        })
                    },
                    Type::Box(elem) => {
                        RustType::Box(Box::new(elem.unwrap(rust_types, user_defined).1))
                    },
                    Type::Str => {
                        rust_types.str = true;
                        RustType::Str
                    },
                    Type::String => {
                        rust_types.string = true;
                        RustType::String
                    },
                    Type::Slice(elem) => {
                        rust_types.slice = true;
                        RustType::Slice(Box::new(elem.unwrap(rust_types, user_defined).1))
                    },
                    Type::Array(_) => RustType::Unsupported,
                    Type::Vec(elem) => {
                        rust_types.vec = true;
                        RustType::Vec(Box::new(elem.unwrap(rust_types, user_defined).1))
                    },
                    Type::UserDefined(ident) => {
                        user_defined.insert(ident.to_string());
                        RustType::UserDefined(ident.to_string())
                    },
                    Type::Tuple(elems) => {
                        rust_types.tuple = true;
                        let elems = elems.iter().map(|elem| {
                            elem.unwrap(rust_types, user_defined).1
                        }).collect();
                        RustType::Tuple(elems)
                    },
                    Type::Unsupported(_) => RustType::Unsupported,
                    _ => unreachable!()
                };
                (FfiType::Pointer, TsType::Rust(rust_type))
            }
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

#[rustfmt::skip]
impl ToTokens for TsType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            TsType::Void    => quote! { void },
            TsType::Boolean => quote! { boolean },
            TsType::Number  => quote! { number },
            TsType::BigInt  => quote! { bigint },
            TsType::String  => quote! { string },
            TsType::TypedArray(typed_array) => typed_array.to_token_stream(),
            TsType::Array(elem) => quote! { #elem[] },
            TsType::Tuple(elems) => quote! { [#(#elems),*] },
            TsType::Rust(rust_type) => rust_type.to_token_stream(),
        });
    }
}

#[rustfmt::skip]
impl ToTokens for TypedArray {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            TypedArray::Uint8Array   => quote! { Uint8Array },
            TypedArray::Uint16Array  => quote! { Uint16Array },
            TypedArray::Uint32Array  => quote! { Uint32Array },
            TypedArray::Uint64Array  => quote! { Uint64Array },
            TypedArray::Int8Array    => quote! { Int8Array },
            TypedArray::Int16Array   => quote! { Int16Array },
            TypedArray::Int32Array   => quote! { Int32Array },
            TypedArray::Int64Array   => quote! { Int64Array },
            TypedArray::Float32Array => quote! { Float32Array },
            TypedArray::Float64Array => quote! { Float64Array },
        });
    }
}



impl ToTokens for RustType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            RustType::Char => quote! { RustChar },
            RustType::FnPointer(sig) => quote! { RustFn<#sig> }, // is this a good idea?
            RustType::Pointer(TypeReference { mut_, elem }) => {
                if *mut_ {
                    quote! { RustPtrMut<#elem> }
                } else {
                    quote! { RustPtr<#elem> }
                }
            },
            RustType::Reference(TypeReference { mut_, elem }) => {
                if *mut_ {
                    quote! { RustRefMut<#elem> }
                } else {
                    quote! { RustRef<#elem> }
                }
            },
            RustType::Box(elem) => quote! { RustBox<#elem> },
            RustType::Str => quote! { RustStr },
            RustType::String => quote! { RustString },
            RustType::Slice(elem) => quote! { RustSlice<#elem> },
            RustType::Vec(elem) => quote! { RustVec<#elem> },
            RustType::Tuple(elem) => quote! { RustTuple<#(#elem),*> },
            RustType::UserDefined(ident) => quote! { #ident },
            RustType::Unsupported => quote! { Deno.PointerObject | null },
        });
    }
}
