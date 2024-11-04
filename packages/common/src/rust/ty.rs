use crate::rust::util::*;

/* ---------------------------------------------------------------------------- */

// MARK: type api

// [!WARNING]: The type API is UNSTABLE
// It is prone to change as support for certain types may be added/changed,
// or refactored as a consequence of a new discovery, optimization, or change in
// Rust's syntax

#[rustfmt::skip]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Type {

    // MARK: primitives

    // [!TODO] develop framework for enforcing/analyzing ffi safety and provide
    // roadmap for this project based on the framework

    // READINGS

    // Unsafe Rust and Undefined Behavior
    // https://doc.rust-lang.org/nomicon/index.html

    // FFI Safety Guidelines
    // https://anssi-fr.github.io/rust-guide/07_ffi.html

    /*------------------ PRIMITIVE TYPES ------------------*/

    /// Explit unit return type ()
    #[default]
    Void,

    // These are trivial types that don't need to be decomposed
    // and are FFI-safe

    /// Numeric types
    /// u8..u64, i8..i64, usize, isize, f32/f64
    Numeric(TypeNumeric),



    /*------------------ BUILT-IN TYPES ------------------*/

    // These are also trivial types, but are not necessarily FFI-safe

    /// A boolean type with two valid values, 0 and 1
    ///
    /// It is stored as a one-byte structure thus having 256 possible
    /// representations. Because of this, it is not ffi robust since checking
    /// whether it is valid depends on the language (i.e. does the languange
    /// enforce `0000 0000` and `0000 0001`, or does it permit values like
    /// `1111 1111`?)
    Bool,

    /// The `char` type has the structure of a u32, but Rust enforces a check
    /// for this type to be a valid unicode scalar value. Some character
    /// decoding may be necessary between language contexts
    Char,

    /// *const T, *mut T
    Pointer(TypeReference),

    /// &T, &mut T
    Reference(TypeReference),

    /// fn(usize) -> ()
    ///
    /// This is passed as an opaque function pointer. In the future, a utility will
    /// be made that enforces against function signatures
    FnPointer(syn::TypeBareFn),

    /// Box<T>
    ///
    /// Box is just a smart pointer
    Box(Box<Type>),

    /* -------------------------------------------- */

    // MARK: aggregates

    // These are non-trivial types, they must be decomposed to be passed between
    // language contexts, and are not necessarily FFI-safe

    /// A string slice `str`.
    /// It must be mutable and should only be used as a unidirectional data
    /// passing mechanism. For strings that expect to be modified in both
    /// contexts, use the `String` type
    Str,

    /// String
    String,

    /// [T]
    Slice(Box<Type>),

    /// Fixed size Array [T; n]
    Array(TypeArray),

    /// Vec<T>
    Vec(Box<Type>),


    /// CustomType
    ///
    /// User-defined types should have some enforcement of being only defined
    /// locally to the crate it is used in
    ///
    /// Require used to annotate structs that define the type to implement some
    /// trait checking mechanism
    UserDefined(Ident),

    /// A tuple type (A, B, C)
    ///
    /// Tuples should be avoided as each tuple type is interpreted as a unique
    /// tuple identity.
    ///
    /// This means that for every unique tuple type, the code
    /// generator will generate monomorphized functions for accessing the
    /// elements of a tuple, and map each tuple type into a vtable for enforcing
    /// type safety and which valid interaces or symbols they may be used with
    /// as a parameter or result type

    ///
    /// [!ISSUE] Implementation for tuple type
    /// ---
    ///
    /// To allow users to interact with each element of a tuple, some mechanism
    /// should be created that interprets a tuple and creates a dereferencing
    /// operation to return each element
    ///
    /// However, if this were to be done on the procedural macro side, there
    /// would be redundant instances or implementations of this tuple element
    /// interaction interace
    ///
    /// ```ignore
    /// #[deno_bindgen]
    /// fn create_str(tup: (*mut u8, usize) ) -> *mut (*mut u8, usize)
    /// // generated
    /// extern "C" fn create_str_tup1(tup: (*mut u8, usize) ) -> *mut u8
    /// extern "C" fn create_str_tup2(tup: (*mut u8, usize) ) -> usize
    ///
    /// #[deno_bindgen]
    /// fn create_buffer(tup: (*mut u8, usize) ) -> *mut (*mut u8, usize)
    /// // generated
    /// extern "C" fn create_buffer_tup1(tup: (*mut u8, usize) ) -> *mut u8
    /// extern "C" fn create_buffer_tup2(tup: (*mut u8, usize) ) -> usize
    /// ```
    ///
    /// Notice how implementing this as a procedural macro would create
    /// redundant symbols, which could bloat the user's code. Procedural macros
    /// cannot identify whether there are other functions that take the same
    /// arity of that tuple throughout the rest of the code
    ///
    /// To avoid this, we need to implement some mechanism that reuses functions
    /// or for interning. But to achieve this, we need to have visibility
    /// throughout the entire source code, which is not possible in the context
    /// of a procedural macro
    ///
    /// One approach for this is introducing an entirely new preprocessing step
    /// which sits between the final macro expansion step and the actual
    /// compilation process by rustc.
    ///
    /// This process will expand the user's source code, do some AST parsing to
    /// identify unqiue tuples of the same tuple arity to a type table, assign
    /// them a mangled type name or identity, and inserts the tuple utility
    /// symbols automatically. This transformed source code may then be passed
    /// to rustc directly as an input for the rest of the compilation process
    Tuple(Vec<Type>),

    /// A type currently unsupported or unrecognized by the FFI implementation,
    /// or has no orthogonal representation in the target language. It is simply
    /// stored on the memory as an opaque pointer value, without an interaction
    /// mechanism in the target language context
    Unsupported(syn::Type),
}

// STABLE
// Likely won't change in the near future
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TypeNumeric {
    U8, // numeric types are ffi-robust
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeReference {
    pub mut_: bool,
    pub elem: Box<Type>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeArray {
    pub elem: Box<Type>,
    pub len:  usize,
}


/* ---------------------------------------------------------------------------- */

// MARK: parse

impl Type {
    /// Parse a Rust type from a stream of Rust source code, at a place where a
    /// type is expected, specifically inside a function signature's inputs and
    /// output
    ///
    /// `self_ty` is an optional argument that should be provided if the type is
    /// being parsed from within an associated function of an `impl` block
    pub fn parse(input: ParseStream, self_ty: Option<&Ident>) -> Result<Self> {
        // RECEVIER TYPES self &self &mut self
        if input.peek(Token![self]) {
            if let Some(self_ty) = self_ty {
                input.parse::<Token![self]>()?;
                return Ok(Type::UserDefined(self_ty.clone()));
            } else {
                return Err(input.error(
                    "unexpected receiver: `self` parameter may only appear in associated functions of trait or implement blocks"
                ));
            }
        }
        if input.peek(Token![Self]) {
            if let Some(self_ty) = self_ty {
                input.parse::<Token![Self]>()?;
                return Ok(Type::UserDefined(self_ty.clone()));
            } else {
                return Err(input.error(
                    "unknown `Self` type. `Self` may only appear in associated functions of trait or implement blocks"
                ));
            }
        }

        // IDENT TYPES String, usize, Vec<T>, CustomType
        if !input.peek(Token![::]) && input.peek(Ident) && !input.peek2(Token![::]) {
            let fork = input.fork();
            let ident = fork.parse::<Ident>()?;

            let ident_str = ident.to_string();
            let mut chars = ident_str.chars();
            let first = chars.next().unwrap();
            let rest = chars.as_str();
            if first == 'u' {
                let numeric_ty = match rest {
                    "8" => Some(TypeNumeric::U8),
                    "16" => Some(TypeNumeric::U16),
                    "32" => Some(TypeNumeric::U32),
                    "64" => Some(TypeNumeric::U64),
                    "size" => Some(TypeNumeric::Usize),
                    _ => None,
                };
                if let Some(numeric_ty) = numeric_ty {
                    input.advance_to(&fork);
                    return Ok(Type::Numeric(numeric_ty));
                }
            } else if first == 'i' {
                let numeric_ty = match rest {
                    "8" => Some(TypeNumeric::I8),
                    "16" => Some(TypeNumeric::I16),
                    "32" => Some(TypeNumeric::I32),
                    "64" => Some(TypeNumeric::I64),
                    "size" => Some(TypeNumeric::Isize),
                    _ => None,
                };
                if let Some(numeric_ty) = numeric_ty {
                    input.advance_to(&fork);
                    return Ok(Type::Numeric(numeric_ty));
                }
            } else if first == 'f' {
                let numeric_ty = match rest {
                    "32" => Some(TypeNumeric::F32),
                    "64" => Some(TypeNumeric::F64),
                    _ => None,
                };
                if let Some(numeric_ty) = numeric_ty {
                    input.advance_to(&fork);
                    return Ok(Type::Numeric(numeric_ty));
                }
            } else if first == 'b' && rest == "ool" {
                input.advance_to(&fork);
                return Ok(Type::Bool);
            } else if first == 'c' && rest == "har" {
                input.advance_to(&fork);
                return Ok(Type::Char);
            } else if first == 's' && rest == "tr" {
                input.advance_to(&fork);
                return Ok(Type::Str);
            } else if first == 'S' && rest == "tring" {
                input.advance_to(&fork);
                return Ok(Type::String);
            } else if first == 'B' && rest == "ox" {
                if let Ok(ty) = GenericArgument::parse_single_with_self_ty(&fork, self_ty) {
                    input.advance_to(&fork);
                    return Ok(Type::Box(Box::new(ty)));
                }
            } else if first == 'V' && rest == "ec" {
                if let Ok(ty) = GenericArgument::parse_single_with_self_ty(&fork, self_ty) {
                    input.advance_to(&fork);
                    return Ok(Type::Vec(Box::new(ty)));
                }
            } else {
                if !input.peek(Token![<]) {
                    input.advance_to(&fork);
                    return Ok(Type::UserDefined(ident));
                }
            }
        }

        // POINTER TYPES *mut
        if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            let ahead = input.lookahead1();
            let mut_ = if ahead.peek(Token![const]) {
                input.parse::<Token![const]>()?;
                false
            } else if ahead.peek(Token![mut]) {
                input.parse::<Token![mut]>()?;
                true
            } else {
                // fail if no `const` or `mut` token was provided
                return Err(ahead.error());
            };
            return Ok(Self::Pointer(TypeReference {
                mut_,
                elem: Box::new(Self::parse(input, self_ty)?),
            }));
        }

        // REFERENCE TYPES &mut
        if input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            if let Some(lifetime) = input.parse::<Option<Lifetime>>()? {
                return Err(Error::new(
                    lifetime.span(),
                    "generic parameters and lifetimes are not supported",
                ));
            }
            let mut_ = input.parse::<Option<Token![mut]>>()?.is_some();

            return Ok(Self::Reference(TypeReference {
                mut_,
                elem: Box::new(Self::parse(input, self_ty)?),
            }));
        }

        // FUNCTION POINTERS fn(usize) -> ()
        if input.peek(Token![fn]) || input.peek(Token![unsafe]) || input.peek(Token![extern]) {
            return Ok(Self::FnPointer(input.parse()?));
        }


        // Slice [T] or Array [T; n]
        if input.peek(Bracket) {
            let content;
            bracketed!(content in input);
            let elem = Self::parse(&content, self_ty)?;
            if content.peek(Token![;]) {
                content.parse::<Token![;]>()?;
                if content.peek(LitInt) {
                    let len = content.parse::<LitInt>()?.base10_parse()?;
                    return Ok(Self::Array(TypeArray {
                        elem: Box::new(elem),
                        len,
                    }));
                } else if let Ok(expr) = content.parse::<Expr>() {
                    // [!TODO] use a tool like miri to evaluate constant expressions?
                    return Err(Error::new(expr.span(), "unsupported expression\nnote: constant expressions cannot be evaluated. please provide an integer literal"));
                } else {
                    return Err(content.error("expected integer literal"));
                }
            } else {
                return Ok(Self::Slice(Box::new(elem)));
            }
        }

        // UNIT () OR TUPLE TYPES (A, B, C)
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            if content.is_empty() {
                return Ok(Self::Void);
            }
            let mut elems = Vec::new();
            loop {
                elems.push(Self::parse(&content, self_ty)?);
                if !content.peek(Token![,]) {
                    break;
                }
                content.parse::<Token![,]>()?;
            }
            return Ok(Self::Tuple(elems));
        }


        let mut ty: syn::Type = input.parse()?;

        #[cfg(feature = "macro")]
        {
            let diag = diag_warning!(ty, "unsupported type");
            let diag = diag.note("this type will be converted into an opaque pointer object and will appear as an `Unsupported` type, which may not be helpful");
            let diag = match &ty {
                syn::Type::Path(_) => diag.help("consider scoping this type path with a `use` statement"),
                _ => diag.help("consider wrapping this type behind a custom type to give it a more descriptive name, or put this type behind a reference or smart pointer"),
            };
            diag.emit();
        }

        if let Some(self_ty) = self_ty {
            TransformSelfType::transform(&mut ty, self_ty);
        }

        Ok(Self::Unsupported(ty))
    }

    pub fn is_self_ty(&self, self_ty: &Ident) -> bool {
        match self {
            Self::UserDefined(ty) => {
                if ty.to_string() == self_ty.to_string() {
                    true
                } else {
                    false
                }
            },
            _ => false,
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: helpers

pub type GenericArgument = Type;

impl GenericArgument {
    // parse a single generic argument that is a concrete type
    // i.e. Box<usize>
    pub fn parse_single_with_self_ty(
        input: ParseStream,
        self_ty: Option<&Ident>,
    ) -> Result<GenericArgument> {
        input.parse::<Option<Token![::]>>()?;
        input.parse::<Token![<]>()?;
        // parse a generic argument
        let fork = input.fork();
        if let Ok(ty) = Type::parse(&fork, self_ty) {
            input.advance_to(&fork);
            input.parse::<Token![>]>()?;
            Ok(ty)
        } else {
            Err(fork.error("unsupported generic argument"))
        }
    }

    // [!TODO] handle types with multiple generic arguments
    // might be used in the future for types like HashMap<K, V>
    pub fn parse_multiple(
        input: ParseStream,
        self_ty: Option<&Ident>,
    ) -> Result<Vec<GenericArgument>> {
        input.parse::<Option<Token![::]>>()?;
        input.parse::<Token![<]>()?;
        let mut args = Vec::new();
        let fork = input.fork();
        loop {
            if fork.peek(Token![>]) {
                break;
            }
            if let Ok(ty) = Type::parse(&fork, self_ty) {
                args.push(ty);
            } else {
                return Err(fork.error("unsupported generic argument"));
            }
            if fork.peek(Token![>]) {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        input.advance_to(&fork);
        Ok(args)
    }
}

/// there is no way to access a rust 'object's symbols outside rust. we need to
/// create a shim that bridges invocations for an associated function
/// http://jakegoulding.com/rust-ffi-omnibus/objects/
///
/// to help with this, a syntax node recurser is defined here. it will transform
/// all instances of Ident("Self") into their actual resolved names
/// `Ident(self_ty)`. used mainly in implement blocks where creating ffi shims
/// aren't allowed to take `Self` parameters outside an impl context
struct TransformSelfType<'a> {
    self_ty: &'a Ident,
}

impl<'a> VisitMut for TransformSelfType<'a> {
    fn visit_ident_mut(&mut self, i: &mut proc_macro2::Ident) {
        if i.to_string().as_str() == "Self" {
            *i = self.self_ty.clone();
        }
        syn::visit_mut::visit_ident_mut(self, i);
    }
}

impl<'a> TransformSelfType<'a> {
    pub fn transform(target: &mut syn::Type, self_ty: &'a Ident) {
        let mut transformer = TransformSelfType { self_ty };
        transformer.visit_type_mut(target);
    }
}

/* -------------------------------------------------------------------------- */

// MARK: parse tests

#[cfg(test)]
mod parse_tests {
    use super::*;

    impl Parse for Type {
        /// Implementation of `Parse` for `Type`. This is only used for
        /// debugging purposes
        fn parse(input: ParseStream) -> Result<Self> {
            let self_ty = if let Some(lit_str) = input.parse::<Option<LitStr>>()? {
                input.parse::<Token![,]>()?;
                Some(format_ident!("{}", lit_str.value()))
            } else {
                None
            };

            Type::parse(input, self_ty.as_ref())
        }
    }

    #[test]
    fn test_self_transformer() {
        let mut ty: syn::Type = syn::parse_quote!( HashMap<Self, Vec<Box<Self>>>);
        println!("from {}", ty.to_token_stream().to_string());
        TransformSelfType::transform(&mut ty, &format_ident!("CustomType"));
        println!("to {}", ty.to_token_stream().to_string());
        assert_eq!(
            ty,
            syn::parse_quote!( HashMap<CustomType, Vec<Box<CustomType>>> )
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_numerics() {
        dbg_assert!(Type::Numeric(TypeNumeric::U8),    parse_quote!(Type, u8));
        dbg_assert!(Type::Numeric(TypeNumeric::U16),   parse_quote!(Type, u16));
        dbg_assert!(Type::Numeric(TypeNumeric::U32),   parse_quote!(Type, u32));
        dbg_assert!(Type::Numeric(TypeNumeric::U64),   parse_quote!(Type, u64));
        dbg_assert!(Type::Numeric(TypeNumeric::Usize), parse_quote!(Type, usize));
        dbg_assert!(Type::Numeric(TypeNumeric::I8),    parse_quote!(Type, i8));
        dbg_assert!(Type::Numeric(TypeNumeric::I16),   parse_quote!(Type, i16));
        dbg_assert!(Type::Numeric(TypeNumeric::I32),   parse_quote!(Type, i32));
        dbg_assert!(Type::Numeric(TypeNumeric::I64),   parse_quote!(Type, i64));
        dbg_assert!(Type::Numeric(TypeNumeric::Isize), parse_quote!(Type, isize));
        dbg_assert!(Type::Numeric(TypeNumeric::F32),   parse_quote!(Type, f32));
        dbg_assert!(Type::Numeric(TypeNumeric::F64),   parse_quote!(Type, f64));
    }

    // exhaustively checks parsing of supported types
    #[test]
    fn test_primitives() {
        dbg_assert!(Type::Void, parse_quote!(Type, ()));
        dbg_assert!(Type::Bool, parse_quote!(Type, bool));
        dbg_assert!(Type::Char, parse_quote!(Type, char));
    }

    #[test]
    fn test_pointers() {
        dbg_assert!(
            Type::Pointer(TypeReference {
                mut_: false,
                elem: Box::new(Type::Numeric(TypeNumeric::U8)),
            }),
            parse_quote!(Type, *const u8)
        );
        dbg_assert!(
            Type::Pointer(TypeReference {
                mut_: true,
                elem: Box::new(Type::Numeric(TypeNumeric::U8)),
            }),
            parse_quote!(Type, *mut u8)
        );
        dbg_assert!(
            Type::Reference(TypeReference {
                mut_: false,
                elem: Box::new(Type::Numeric(TypeNumeric::U8)),
            }),
            parse_quote!(Type, &u8)
        );
        dbg_assert!(
            Type::Reference(TypeReference {
                mut_: true,
                elem: Box::new(Type::Numeric(TypeNumeric::U8)),
            }),
            parse_quote!(Type, &mut u8)
        );
        dbg_assert!(
            Type::FnPointer({
                let ty: syn::Type = syn::parse_quote!(fn(u8) -> u8);
                match ty {
                    syn::Type::BareFn(type_bare_fn) => type_bare_fn,
                    _ => panic!("unexpected error while parsing type"),
                }
            }),
            parse_quote!(Type, fn(u8) -> u8)
        );
        dbg_assert!(
            Type::Box(Box::new(Type::Numeric(TypeNumeric::U8))),
            parse_quote!(Type, Box<u8>)
        );
    }

    #[test]
    fn test_collections() {
        dbg_assert!(Type::Str, parse_quote!(Type, str));
        dbg_assert!(
            Type::Reference(TypeReference {
                mut_: true,
                elem: Box::new(Type::Str),
            }),
            parse_quote!(Type, &mut str)
        );
        dbg_assert!(Type::String, parse_quote!(Type, String));
        dbg_assert!(
            Type::Slice(Box::new(Type::Numeric(TypeNumeric::U8))),
            parse_quote!(Type, [u8])
        );
        dbg_assert!(
            Type::Reference(TypeReference {
                mut_: true,
                elem: Box::new(Type::Slice(Box::new(Type::Numeric(TypeNumeric::U8)))),
            }),
            parse_quote!(Type, &mut [u8])
        );
        dbg_assert!(
            Type::Array(TypeArray {
                elem: Box::new(Type::Numeric(TypeNumeric::U8)),
                len:  8,
            }),
            parse_quote!(Type, [u8; 8])
        );
        dbg_assert!(
            Type::Vec(Box::new(Type::Box(Box::new(Type::Numeric(
                TypeNumeric::U8
            ))))),
            parse_quote!(Type, Vec<Box<u8>>)
        );
    }

    #[test]
    fn test_tuple() {
        dbg_assert!(
            Type::Tuple(vec![Type::Numeric(TypeNumeric::U8), Type::String,]),
            parse_quote!(Type, (u8, String))
        );
        dbg_assert!(
            Type::Tuple(vec![
                Type::Numeric(TypeNumeric::U8),
                Type::Box(Box::new(Type::Tuple(vec![
                    Type::Numeric(TypeNumeric::Usize),
                    Type::Numeric(TypeNumeric::U8)
                ]))),
                Type::String,
                Type::Reference(TypeReference {
                    mut_: true,
                    elem: Box::new(Type::Numeric(TypeNumeric::U8)),
                })
            ]),
            parse_quote!(Type, (u8, Box<(usize, u8)>, String, &mut u8))
        );
    }

    #[test]
    fn test_user_defined() {
        dbg_assert!(
            Type::UserDefined(format_ident!("SomeOtherType")),
            parse_quote!(Type, SomeOtherType)
        );
        dbg_assert!(
            Type::UserDefined(format_ident!("SomeOtherType")),
            parse_quote!(Type, "CustomType", SomeOtherType)
        );
    }

    #[test]
    fn test_self_receivers() {
        dbg_assert!(
            Type::UserDefined(format_ident!("CustomType")),
            parse_quote!(Type, "CustomType", CustomType)
        );
        dbg_assert!(
            Type::UserDefined(format_ident!("CustomType")),
            parse_quote!(Type, "CustomType", Self)
        );
        dbg_assert!(
            Type::Reference(TypeReference {
                mut_: false,
                elem: Box::new(Type::UserDefined(format_ident!("CustomType"))),
            }),
            parse_quote!(Type, "CustomType", &Self)
        );
        dbg_assert!(
            Type::Reference(TypeReference {
                mut_: true,
                elem: Box::new(Type::UserDefined(format_ident!("CustomType"))),
            }),
            parse_quote!(Type, "CustomType", &mut Self)
        );
    }

    #[test]
    #[should_panic]
    fn test_self_receiver_fail() {
        parse_quote!(Type, Self);
    }

    #[test]
    fn test_unsupported_path() {
        dbg_assert!(
            Type::Unsupported(syn::parse_quote!(std::io::File)),
            parse_quote!(Type, std::io::File)
        );
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

#[rustfmt::skip]
impl ToTokens for TypeNumeric {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            TypeNumeric::U8    => quote! { u8 },
            TypeNumeric::U16   => quote! { u16 },
            TypeNumeric::U32   => quote! { u32 },
            TypeNumeric::U64   => quote! { u64 },
            TypeNumeric::Usize => quote! { usize },
            TypeNumeric::I8    => quote! { i8 },
            TypeNumeric::I16   => quote! { i16 },
            TypeNumeric::I32   => quote! { i32 },
            TypeNumeric::I64   => quote! { i64 },
            TypeNumeric::Isize => quote! { isize },
            TypeNumeric::F32   => quote! { f32 },
            TypeNumeric::F64   => quote! { f64 },
        };

        tokens.extend(iter);
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let iter = match self {
            Type::Void => unreachable!("attempted to print unit `()` type"),
            Type::Numeric(type_numeric) => type_numeric.to_token_stream(),
            Type::Bool => quote! { bool },
            Type::Char => quote! { char },
            Type::Pointer(TypeReference { mut_, elem }) => match mut_ {
                true => quote! { *mut #elem },
                false => quote! { *const #elem },
            },
            Type::Reference(TypeReference { mut_, elem }) => match mut_ {
                true => quote! { &mut #elem },
                false => quote! { &#elem },
            },
            Type::FnPointer(type_bare_fn) => type_bare_fn.to_token_stream(),
            Type::Box(elem) => quote! { Box<#elem> },
            Type::Str => quote! { str },
            Type::String => quote! { String },
            Type::Slice(elem) => quote! { [#elem] },
            Type::Array(TypeArray { elem, len }) => {
                let len = LitInt::new(len.to_string().as_str(), Span::mixed_site());
                quote! { [#elem; #len] }
            },
            Type::Vec(elem) => quote! { Vec<#elem> },
            Type::UserDefined(ident) => ident.to_token_stream(),
            Type::Tuple(elems) => quote! { ( #(#elems),* ) },
            Type::Unsupported(ty) => ty.to_token_stream(),
        };

        tokens.extend(iter);
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[rustfmt::skip]
#[cfg(test)]
mod print_tests {
    use super::*;

    macro_rules! test_print {
        ( $($tt:tt)* ) => {
            dbg_assert!(parse_quote!(Type, $($tt)*).to_token_stream().to_string(), stringify!($($tt)*))
        };
    }

    #[test]
    #[should_panic]
    fn test_void() {
        // void type should never be printed
        // as a parameter, i.e. (arg0: ()) or return type fn(...) -> ()
        // this is useless and should be omitted
        test_print!(());
    }

    #[test]
    fn test_primitives() {
        test_print!(u8);
        test_print!(u16);
        test_print!(u32);
        test_print!(u64);
        test_print!(usize);
        test_print!(i8);
        test_print!(i16);
        test_print!(i32);
        test_print!(i64);
        test_print!(isize);
        test_print!(f32);
        test_print!(f64);
        test_print!(bool);
        test_print!(char);
    }

    #[test]
    fn test_pointers() {
        test_print!(* mut u8);
        test_print!(* const u8);
        test_print!(& u8);
        test_print!(& mut u8);
        dbg_assert!(
            parse_quote!(Type, fn(u8) -> u8)
                .to_token_stream()
                .to_string(),
            "fn (u8) -> u8"
        );
        test_print!(Box < u8 >);
    }

    #[test]
    fn test_collections() {
        test_print!(str);
        test_print!(& mut str);
        test_print!(String);
        test_print!([u8]);
        test_print!(& mut [u8]);
        dbg_assert!(
            parse_quote!(Type, [u8; 8])
                .to_token_stream()
                .to_string(),
            "[u8 ; 8]"
        );
        test_print!(Vec < Box < u8 > >);
    }

    #[test]
    fn test_tuple() {
        dbg_assert!(
            parse_quote!(Type, (u8, String))
                .to_token_stream()
                .to_string(),
            "(u8 , String)"
        );
        dbg_assert!(
            parse_quote!(Type, (u8, Box<(usize, u8)>, String, &mut u8))
                .to_token_stream()
                .to_string(),
            "(u8 , Box < (usize , u8) > , String , & mut u8)"
        );
    }
}
