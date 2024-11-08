use std::collections::BTreeSet;

use crate::deno::util::*;
use crate::deno::{ClassDefs, FfiInterface, FfiType, TsFormat, TsModule};
use crate::rust::{Type, TypeNumeric};

/* -------------------------------------------------------------------------- */

// MARK: rust type

#[derive(Clone, Debug, PartialEq)]
pub enum RustTypeNumeric {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
    U64,
    I64,
    Usize,
    Isize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RustType {
    Void,
    Numeric(RustTypeNumeric),
    Boolean,
    Char,
    FnPtr(String), // pointer object containing actual reference value and signature value
    Ptr(Box<RustType>),
    PtrMut(Box<RustType>),
    Ref(Box<RustType>),
    RefMut(Box<RustType>),
    Box(Box<RustType>),
    Str,
    String,
    Slice(Box<RustType>),
    Vec(Box<RustType>),
    Tuple(Vec<RustType>),
    UserDefined(Ident),
    Unsupported, // generic deno pointer object
}

/* -------------------------------------------------------------------------- */

// MARK:  transform

impl Type {
    pub fn unwrap(self, module: &mut TsModule) -> (FfiType, RustType) {
        let TsModule {
            type_defs,
            user_defs,
            ..
        } = module;


        fn match_str_or_slice(
            elem: Box<Type>,
            module: &mut TsModule,
            fallback: &dyn Fn(Box<Type>, &mut TsModule) -> RustType,
        ) -> RustType {
            match *elem {
                Type::Str => {
                    module.type_defs.insert(RustTypeList::Str);
                    RustType::Str
                },
                Type::Slice(elem) => {
                    module.type_defs.insert(RustTypeList::Slice);
                    let elem = Box::new(elem.unwrap(module).1);
                    RustType::Slice(elem)
                },
                _ => fallback(elem, module),
            }
        }

        match self {
            Type::Void => (FfiType::Void, RustType::Void),
            Type::Numeric(type_numeric) => match type_numeric {
                TypeNumeric::U8 => {
                    type_defs.insert(RustTypeList::U8);
                    (FfiType::U8, RustType::Numeric(RustTypeNumeric::U8))
                },
                TypeNumeric::U16 => {
                    type_defs.insert(RustTypeList::U16);
                    (FfiType::U16, RustType::Numeric(RustTypeNumeric::U16))
                },
                TypeNumeric::U32 => {
                    type_defs.insert(RustTypeList::U32);
                    (FfiType::U32, RustType::Numeric(RustTypeNumeric::U32))
                },
                TypeNumeric::I8 => {
                    type_defs.insert(RustTypeList::I8);
                    (FfiType::I8, RustType::Numeric(RustTypeNumeric::I8))
                },
                TypeNumeric::I16 => {
                    type_defs.insert(RustTypeList::I16);
                    (FfiType::I16, RustType::Numeric(RustTypeNumeric::I16))
                },
                TypeNumeric::I32 => {
                    type_defs.insert(RustTypeList::I32);
                    (FfiType::I32, RustType::Numeric(RustTypeNumeric::I32))
                },
                TypeNumeric::F32 => {
                    type_defs.insert(RustTypeList::F32);
                    (FfiType::F32, RustType::Numeric(RustTypeNumeric::F32))
                },
                TypeNumeric::F64 => {
                    type_defs.insert(RustTypeList::F64);
                    (FfiType::F64, RustType::Numeric(RustTypeNumeric::F64))
                },
                TypeNumeric::U64 => {
                    type_defs.insert(RustTypeList::U64);
                    (FfiType::U64, RustType::Numeric(RustTypeNumeric::U64))
                },
                TypeNumeric::I64 => {
                    type_defs.insert(RustTypeList::I64);
                    (FfiType::I64, RustType::Numeric(RustTypeNumeric::I64))
                },
                TypeNumeric::Usize => {
                    type_defs.insert(RustTypeList::Usize);
                    (FfiType::Usize, RustType::Numeric(RustTypeNumeric::Usize))
                },
                TypeNumeric::Isize => {
                    type_defs.insert(RustTypeList::Isize);
                    (FfiType::Isize, RustType::Numeric(RustTypeNumeric::Isize))
                },
            },
            Type::Bool => (FfiType::Bool, RustType::Boolean),
            Type::Char => {
                type_defs.insert(RustTypeList::Char);
                (FfiType::U32, RustType::Char)
            },
            Type::FnPtr(type_bare_fn) => {
                type_defs.insert(RustTypeList::FnPtr);
                (
                    FfiType::FnPointer,
                    RustType::FnPtr(type_bare_fn.to_token_stream().to_string()),
                )
            },
            rest => (FfiType::Pointer, match rest {
                Type::Ptr(elem) => match_str_or_slice(elem, module, &|elem, module| {
                    module.type_defs.insert(RustTypeList::Ptr);
                    let elem = Box::new(elem.unwrap(module).1);
                    RustType::Ptr(elem)
                }),
                Type::PtrMut(elem) => match_str_or_slice(elem, module, &|elem, module| {
                    module.type_defs.insert(RustTypeList::PtrMut);
                    let elem = Box::new(elem.unwrap(module).1);
                    RustType::PtrMut(elem)
                }),
                Type::Ref(elem) => match_str_or_slice(elem, module, &|elem, module| {
                    module.type_defs.insert(RustTypeList::Ref);
                    let elem = Box::new(elem.unwrap(module).1);
                    RustType::Ref(elem)
                }),
                Type::RefMut(elem) => match_str_or_slice(elem, module, &|elem, module| {
                    module.type_defs.insert(RustTypeList::RefMut);
                    let elem = Box::new(elem.unwrap(module).1);
                    RustType::RefMut(elem)
                }),
                Type::Box(elem) => match_str_or_slice(elem, module, &|elem, module| {
                    module.type_defs.insert(RustTypeList::Box);
                    RustType::Box(Box::new(elem.unwrap(module).1))
                }),
                Type::Str => {
                    type_defs.insert(RustTypeList::Str);
                    RustType::Str
                },
                Type::String => {
                    type_defs.insert(RustTypeList::String);
                    RustType::String
                },
                Type::Slice(elem) => {
                    type_defs.insert(RustTypeList::Slice);
                    RustType::Slice(Box::new(elem.unwrap(module).1))
                },
                Type::Array(_) => {
                    type_defs.insert(RustTypeList::Unsupported);
                    RustType::Unsupported
                },
                Type::Vec(elem) => {
                    type_defs.insert(RustTypeList::Vec);
                    RustType::Vec(Box::new(elem.unwrap(module).1))
                },
                Type::UserDefined(ident) => {
                    user_defs.insert(ident.clone());
                    RustType::UserDefined(ident)
                },
                Type::Tuple(elems) => {
                    type_defs.insert(RustTypeList::Tuple);
                    let mut tup_elems = Vec::new();
                    for elem in elems {
                        tup_elems.push(elem.unwrap(module).1);
                    }
                    RustType::Tuple(tup_elems)
                },
                Type::Unsupported(_) => {
                    type_defs.insert(RustTypeList::Unsupported);
                    RustType::Unsupported
                },
                _ => unreachable!(),
            }),
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: transform tests

#[cfg(test)]
mod transform_tests {
    use super::*;
    use crate::{dbg_assert, parse_quote};

    macro_rules! unwrap_type {
        ($($tt:tt)*) => {{
            let mut module = TsModule::default();
            let actual = Type::unwrap(parse_quote!(Type, $($tt)*), &mut module);
            actual.1
        }};
    }

    #[test]
    fn test_ts_type() {
        dbg_assert!(unwrap_type!(()), RustType::Void);
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print

#[rustfmt::skip]
impl ToTokens for RustTypeNumeric {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            RustTypeNumeric::U8    => quote! { RustU8 },
            RustTypeNumeric::U16   => quote! { RustU16 },
            RustTypeNumeric::U32   => quote! { RustU32 },
            RustTypeNumeric::I8    => quote! { RustI8 },
            RustTypeNumeric::I16   => quote! { RustI16 },
            RustTypeNumeric::I32   => quote! { RustI32 },
            RustTypeNumeric::F32   => quote! { RustF32 },
            RustTypeNumeric::F64   => quote! { RustF64 },
            RustTypeNumeric::U64   => quote! { RustU64 },
            RustTypeNumeric::I64   => quote! { RustI64 },
            RustTypeNumeric::Usize => quote! { RustUsize },
            RustTypeNumeric::Isize => quote! { RustIsize },
        });
    }
}

#[rustfmt::skip]
impl ToTokens for RustType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        fn get_str_or_slice(elem: &Box<RustType>, fallback: TokenStream) -> TokenStream {
            match &**elem {
                RustType::Str => quote! { RustStr },
                RustType::Slice(elem) => quote! { RustSlice<#elem> },
                _ => fallback,
            }
        }

        tokens.extend(match self {
            RustType::Void => quote! { void },
            RustType::Numeric(ts_type_numeric) => ts_type_numeric.to_token_stream(),
            RustType::Boolean => quote! { boolean },
            RustType::Char => quote! { RustChar },
            RustType::FnPtr(sig) => quote! { RustFnPtr<#sig> },
            RustType::Ptr(elem) =>
            get_str_or_slice(elem, quote! { RustPtr<#elem> }),
            RustType::PtrMut(elem) =>
            get_str_or_slice(elem, quote! { RustPtrMut<#elem> }),
            RustType::Ref(elem) =>
            get_str_or_slice(elem, quote! { RustRef<#elem> }),
            RustType::RefMut(elem) =>
            get_str_or_slice(elem, quote! { RustRefMut<#elem> }),
            RustType::Box(elem) =>
            get_str_or_slice(elem, quote! { RustBox<#elem> }),
            RustType::Str => quote! { RustStr },
            RustType::String => quote! { RustString },
            RustType::Slice(elem) => quote! { RustSlice<#elem> },
            RustType::Vec(elem) => quote! { RustVec<#elem> },
            RustType::Tuple(elem) => quote! { RustTuple<[#(#elem),*]> },
            RustType::UserDefined(ident) => quote! { #ident },
            RustType::Unsupported => quote! { RustUnsupportedType },
        });
    }
}

/* -------------------------------------------------------------------------- */

// MARK: user defined

/// Container for user-defined types that do not have implementations or not
/// included in the list of declared structs
#[derive(Clone, Debug, Default)]
pub struct UserDefinedDefs {
    store: BTreeSet<Ident>,
}

impl UserDefinedDefs {
    fn insert(&mut self, ident: Ident) {
        self.store.insert(ident);
    }

    /// removes any user defined type from this list if it matches that from the
    /// list of class definitions
    pub fn dedup(&mut self, class_defs: &ClassDefs) {
        class_defs.store.keys().for_each(|class_name| {
            self.store
                .extract_if(|user_defined| *user_defined == *class_name)
                .for_each(drop);
        });
    }
}

impl ToTokens for UserDefinedDefs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(
            self.store
                .iter()
                .map(|ident| {
                    quote! {

                        export class #ident extends RustPrototype<#ident> {}

                    }
                })
                .collect::<TokenStream>(),
        );
    }
}

/* -------------------------------------------------------------------------- */

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum RustTypeList {
    U8,
    U16,
    U32,
    I8,
    I16,
    I32,
    F32,
    F64,
    U64,
    Usize,
    I64,
    Isize,
    Char,
    FnPtr,
    Ptr,
    PtrMut,
    Ref,
    RefMut,
    Box,
    Str,
    String,
    Slice,
    Vec,
    Tuple,
    Unsupported,
    // types not checked:
    // Void,
    // Boolean,
    // UserDefined,
}

impl ToTokens for RustTypeList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            RustTypeList::U8 => quote! { RustU8 },
            RustTypeList::U16 => quote! { RustU16 },
            RustTypeList::U32 => quote! { RustU32 },
            RustTypeList::I8 => quote! { RustI8 },
            RustTypeList::I16 => quote! { RustI16 },
            RustTypeList::I32 => quote! { RustI32 },
            RustTypeList::F32 => quote! { RustF32 },
            RustTypeList::F64 => quote! { RustF64 },
            RustTypeList::U64 => quote! { RustU64 },
            RustTypeList::Usize => quote! { RustUsize },
            RustTypeList::I64 => quote! { RustI64 },
            RustTypeList::Isize => quote! { RustIsize },
            RustTypeList::Char => quote! { RustChar },
            RustTypeList::FnPtr => quote! { RustFnPtr },
            RustTypeList::Ptr => quote! { RustPtr },
            RustTypeList::PtrMut => quote! { RustPtrMut },
            RustTypeList::Ref => quote! { RustRef },
            RustTypeList::RefMut => quote! { RustRefMut },
            RustTypeList::Box => quote! { RustBox },
            RustTypeList::Str => quote! { RustStr },
            RustTypeList::String => quote! { RustString },
            RustTypeList::Slice => quote! { RustSlice },
            RustTypeList::Vec => quote! { RustVec },
            RustTypeList::Tuple => quote! { RustTuple },
            RustTypeList::Unsupported => quote! { RustUnsupportedType },
        });
    }
}

/* -------------------------------------------------------------------------- */

struct FileStr<'a> {
    store: &'a str,
}
impl<'a> FileStr<'a> {
    fn new(source: &'a str) -> Self {
        Self { store: source }
    }
    /// Extract a string slice matched between a start and end marker and return
    /// the inner module as a String
    fn slice_between(self, start_marker: &'a str, end_marker: &'a str) -> &'a str {
        self.store
            .split(start_marker)
            .nth(1)
            .expect("failed to match a start marker")
            .split(end_marker)
            .next()
            .expect("start marker is missing a closing delimiter")
    }
    /// Find zero or more occurences of a section matched by a start and end
    /// marker and take it out of a string, returning the result after taking
    /// out those sections
    fn slice_out(self, start_marker: &'a str, end_marker: &'a str) -> Vec<&'a str> {
        let mut out = Vec::new();
        let match_count = self.store.matches(start_marker).count();
        if match_count > 0 {
            if match_count == self.store.matches(end_marker).count() {
                let mut rest = self.store;
                for _ in 0..match_count {
                    let (left, right) = rest.split_once(start_marker).unwrap();
                    out.push(left);
                    let (_, right) = right.split_once(end_marker).unwrap();
                    rest = right;
                }
                out.push(rest);
            } else {
                panic!("slice out marker is missing a closing pair");
            }
        } else {
            out.push(self.store);
        }
        out
    }
}

/* -------------------------------------------------------------------------- */

// MARK: rust type defs

/// Helper for generating typescript declarations/representations of standard
/// rust types. These types should not be exposed to downstream external users
/// of the generated ffi library to prevent accidental misuse of the API
#[derive(Clone, Debug, Default)]
pub struct RustTypeDefs {
    store:        BTreeSet<RustTypeList>,
    /// Whether to use the extended variants of the rust type declarations which
    /// include methods for interacting with the underlying rust data structure
    ///
    /// If using extended types, prompt user whether to embed the string utility
    /// functions as a dylib in the user's library or to fetch it remotely
    pub extended: bool,
}

impl RustTypeDefs {
    fn insert(&mut self, ty: RustTypeList) {
        self.store.insert(ty);
    }
}

impl RustTypeDefs {
    /// Print rust types inline
    pub fn print_inline(&self) -> (String, FfiInterface) {
        const CONTENT_START: &'static str = "// <!-- deno-bindgen2-content-start -->\n";
        const CONTENT_END: &'static str = "// <!-- deno-bindgen2-content-end -->\n";
        const ALT_START: &'static str = "// <!-- deno-bindgen2-alt-type-start -->\n";
        const ALT_END: &'static str = "// <!-- deno-bindgen2-alt-type-end -->\n";

        let util_file = include_str!("../../../utils/src/util.ts");
        let core_file = include_str!("../../../utils/src/core.ts");
        let extended_file = include_str!("../../../utils/src/extended.ts");

        let util = FileStr::new(&util_file).slice_between(CONTENT_START, CONTENT_END);
        let core_types = FileStr::new(&core_file).slice_between(CONTENT_START, CONTENT_END);

        let ffi_interface;

        let mut module = String::new();
        module.push_str(util);
        module.push_str(core_types);
        if self.extended {
            let extended_types = FileStr::new(&extended_file).slice_between(ALT_START, ALT_END);
            let extended_types = FileStr::new(extended_types).slice_out(
                "// <!-- deno-bindgen2-ignore-start -->",
                "// <!-- deno-bindgen2-ignore-end -->",
            );
            for slice in extended_types {
                module.push_str(slice);
            }

            let _ffi_interface = <TokenStream as std::str::FromStr>::from_str(
                FileStr::new(&extended_file).slice_between(
                    "    // <!-- deno-bindgen2-ffi-symbols-start -->\n",
                    "    // <!-- deno-bindgen2-ffi-symbols-end -->\n",
                ),
            )
            .expect("failed to parse ffi symbol tokens");

            ffi_interface = syn::parse2(_ffi_interface).expect("failed to parse ffi interface");
        } else {
            let core_alt_types = FileStr::new(&core_file).slice_between(ALT_START, ALT_END);
            module.push_str(core_alt_types);
            ffi_interface = FfiInterface::default();
        }

        (module, ffi_interface)
    }

    /// Print rust types as a new file, along with an import statement
    pub fn print_separate(
        &self,
        mut type_defs: String, // contents of type_defs module output
        type_defs_name: &str,
    ) -> (String, String) {
        let mut type_imports = Vec::new();
        let mut class_imports = Vec::new();

        for ty in &self.store {
            match ty {
                RustTypeList::Char
                | RustTypeList::Box
                | RustTypeList::Str
                | RustTypeList::String
                | RustTypeList::Slice
                | RustTypeList::Vec
                | RustTypeList::Tuple => class_imports.push(ty.to_token_stream()),
                rest => type_imports.push(rest.to_token_stream()),
            };
        }

        if type_imports.is_empty() && class_imports.is_empty() {
            (String::new(), String::new())
        } else {
            class_imports.push(quote! { RustPrototype });

            type_defs.push_str(
                TsFormat::format(
                    quote! {

                        export type { #(#type_imports),* };
                        export { #(#class_imports),* };

                    }
                    .to_string(),
                )
                .as_str(),
            );

            let type_defs_name = format!("./{type_defs_name}");
            let imports = TsFormat::format(
                quote! {
                    import type { #(#type_imports),* } from #type_defs_name;
                    import { #(#class_imports),* } from #type_defs_name;
                }
                .to_string(),
            );

            (type_defs, imports)
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print tests

#[cfg(test)]
mod print_tests {
    use super::*;
    use crate::deno::{FfiLib, TsFormat};

    #[test]
    fn test_print() {}

    #[test]
    fn print_type_def() {
        let mut type_defs = RustTypeDefs::default();
        type_defs.extended = true;
        let (module, ffi_interface) = type_defs.print_inline();
        let mut ffi_lib = FfiLib::default();
        ffi_lib.interface = ffi_interface;

        println!(
            "{}",
            TsFormat::format(ffi_lib.to_token_stream().to_string())
        );
        println!("{module}");
    }
}
