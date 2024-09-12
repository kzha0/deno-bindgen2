use darling::{
    ast::NestedMeta,
    FromMeta,
};
use proc_macro_error::{
    abort,
    emit_error,
    emit_warning,
};
use syn::{
    ItemFn,
    Signature,
    TypePath,
    TypeReference,
    TypeSlice, TypeTuple,
};

use crate::r#mod::*;

#[derive(Default, FromMeta, Debug)]
#[darling(default)]
pub struct MacroArgs {
    optional:     bool,
    non_blocking: bool,
}

// flow of type transformation:
// TokenStream -> ParserItem -> IrItem -> TransformItem -> TokenStream
// singleton -> singleton -> multiple -> multiple -> singleton

pub struct Parser();
impl Parser {
    pub fn from_attr_stream(attr: TokenStream, input: TokenStream) -> ParserItem {
        let macro_args = NestedMeta::parse_meta_list(attr.into())
            .and_then(|meta_list| {
                let mut this = MacroArgs::default();
                for meta in meta_list {
                    let other =
                        MacroArgs::from_list(&*Box::new([meta.clone()])).unwrap_or_else(|err| {
                            abort!(meta, "failed to parse macro attribute: {}", err)
                        });
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

        let item: Item = syn::parse2(input.into()).unwrap_or_else(|err| {
            abort!("failed to parse input code: {}", err);
        });

        ParserItem(macro_args, item)
    }
}

pub struct ParserItem(MacroArgs, Item);
impl ParserItem {
    pub fn parse(self) -> TransformItem {
        let ParserItem(macro_args, item) = self;

        match item {
            Item::Fn(item) => {
                ParserFn::parse(
                    MacroArgsFn {
                        optional:     macro_args.optional,
                        non_blocking: macro_args.non_blocking,
                        _internal:    false,
                    },
                    item,
                )
            },
            Item::Struct(_) => todo!(),
            Item::Impl(_) => todo!(),
            item => {
                // TODO: change this to `emit_error!` and return an unmodified TokenStream
                abort!(
                // emit_error!(
                    item, "unsupported AST item";
                    note = "`deno_bindgen` may only be used with `fn`, `struct`, and `impl` code.";
                );
            },
        }
    }
}

pub struct MacroArgsFn {
    optional:     bool,
    non_blocking: bool,
    _internal:    bool,
}

// These will be reused by ParserImpl for handling associated functions
pub struct ParserFn();
impl ParserFn {
    pub fn parse(fn_args: MacroArgsFn, item: ItemFn) -> TransformItem {
        // Check function signature
        match item.sig {
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

        TransformItem(Item::Fn(item))

        // get a list of inputs and outputs for the original function
        // convert into IrType

        // handle case for ExtendedType
        // get the ReturnType first since this is important

        // if ReturnType must be expanded, create a holder for new functions
        // out_items = Punctuated<ItemFn, Comma>? Vec<ItemFn>

        // match the results into their expansions

        // match String
        // {fn_name}__to_raw (original fn name args) -> *(buffer_ptr, len, cap)
        // {fn}__get_ptr(*_) -> ptr
        // {fn}__get_len(*_) -> len
        // {fn}__get_cap(*_) -> cap

        // the string should now be constructed by Deno from its raw parts. invoke a pointer to deallocate the (ptr, len, cap) tuple values on the heap.
        // note: this does not destroy the actual string because *(ptr) is not the same as ptr
        // TODO: confirm that Deno's pointer API copies the typed array. if so, also deallocate the duplicate String in rust
        // {fn}__drop_raw(*_) -> ()

        // to do this polymorpically, create a trait ToRaw with a function signature:
        // (ExtendedType) -> Vec<ItemFn>??
        // invoke `to_raw` function for each result item, store the tuple into a list of result types
        // create a function generator for each result type

        // if result type is nativetype (i.e. only one return item), still push to a singleton list
    }
}

// mappings for syn::Type to IrType

impl From<syn::Type> for IrType {
    fn from(ty: syn::Type) -> Self {
        match ty {
            syn::Type::Path(TypePath { path, .. }) if path.get_ident().is_some() => {
                match path.get_ident().unwrap().to_string().as_str() {
                    "bool" => IrType::Native(Native::Bool),
                    "u8" => IrType::Native(Native::U8),
                    "i8" => IrType::Native(Native::I8),
                    "u16" => IrType::Native(Native::U16),
                    "i16" => IrType::Native(Native::I16),
                    "u32" => IrType::Native(Native::U32),
                    "i32" => IrType::Native(Native::I32),
                    "u64" => IrType::Native(Native::U64),
                    "i64" => IrType::Native(Native::I64),
                    "usize" => IrType::Native(Native::USize),
                    "isize" => IrType::Native(Native::ISize),
                    "f32" => IrType::Native(Native::F32),
                    "f64" => IrType::Native(Native::F64),

                    // these types are consummable: they should automatically have a deallocator
                    "String" => IrType::NonTrivial(NonTrivial::String),
                    ident => {
                        IrType::NonTrivial(NonTrivial::UserDefined(Box::leak(
                            ident.to_string().into_boxed_str(),
                        )))
                    },
                }
            },
            // Raw pointers and references are different
            // Rust cannot make guarantees about the validity of a reference to memory allocated outside Rust (i.e. from JsLand). There could be cases wherein a Rust function may be passed a reference pointer when the pointed value may have been deallocated in JsLand already
            // This should warn users if consuming or returning a non-Native type as it may cause unsafe memory errors

            // However, as long as references to non-Native types are used in the context of a Ts class constructor, we can safely provide lifetimes for the validity of that reference by limiting its usage and any consuming functions within the methods of a class/object, automatically deallocating whenever it goes out of scope
            syn::Type::Reference(TypeReference {
                mutability, elem, ..
            }) => {
                let mutable = mutability.is_some();
                match *elem {
                    // special case for handling string slice
                    syn::Type::Path(TypePath { path, .. }) if path.get_ident().is_some() => {
                        match path.get_ident().unwrap().to_string().as_str() {
                            // these types are consummable: they should automatically have a deallocator
                            "str" => IrType::NonTrivial(NonTrivial::StringSlice),
                            _ => todo!(),
                        }
                    },
                    // all other cases
                    ty => {
                        let ty = Box::new(IrType::from(ty));
                        if mutability.is_some() {
                            IrType::NonTrivial(NonTrivial::ReferenceMut(ty))
                        } else {
                            IrType::NonTrivial(NonTrivial::Reference(ty))
                        }
                    },
                }
            },
            syn::Type::Tuple(TypeTuple { elems, .. } ) => {
                IrType::NonTrivial(NonTrivial::Tuple(Vec::from_iter(elems.iter().map(|ty| {
                    IrType::from(ty.clone())
                }))))
            },
            // break down into a tuple type of raw parts
            syn::Type::Slice(TypeSlice { elem, .. }) => {
                IrType::NonTrivial(NonTrivial::Slice(Box::new(IrType::from(*elem.clone()))))
            },

            // dangerous raw pointer. if it is part of a top-level item, do not create deallocator as it is assumed that the user knows what they're dealing with
            // if this type is part of an associated function, a deallocator function is automatically created for it
            syn::Type::Ptr(_) => IrType::Native(Native::Pointer),

            ty => IrType::Unsupported(ty),
        }
    }
}

// make a parser for ReturnType here

/*================== STRUCTS AND METHODS/IMPL =================*/

fn parse_impl() {}

fn parse_struct() {}
