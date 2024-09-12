use proc_macro::TokenStream;
use syn::{
    Ident, ItemFn, Pat, PatType

};

// Trivial or fundamental types, a Rust equivalent type that implements the `Copy` trait
// these types may be "trivially" copied by the `std::mem` module without any other special action
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum Native {
    #[default]
    Void,
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    USize,
    ISize,
    F32,
    F64,
    Pointer, // a thin pointer
    Buffer(Box<Native>),  // pointer to a slice
    Function, // function pointer
             // Struct(Box<[Trivial]>), // TODO: Handle struct parameters/return values in the future
}

impl Native {
    pub fn is_numeric(&self) -> bool {
        match &self {
            Native::Pointer
            | Native::Buffer(_)
            | Native::Function => false,
            _ => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum NonTrivial {
    Tuple(Vec<IrType>), // breaks down complex types into their simple raw parts. should handle nested tuples
    String,             // must be converted to string slice
    StringSlice,
    Slice(Box<IrType>),     // must be converted to a (buffer ptr, usize) tupple
    Reference(Box<IrType>), // must be checked for existence of NonTrivial because this is unsafe
    ReferenceMut(Box<IrType>),
    UserDefined(&'static str),
    Receiver(syn::Receiver),
}

// types may be trivial/fundamental, or non-trivial
// the determinant for a trivial type is that it can be copied into memory with memcpy without doing anything
#[derive(Clone, Debug)]
pub enum IrType {
    Native(Native), // these are types directly supported by Deno without further implementation
    NonTrivial(NonTrivial), // types that are not directly supported but can be trasmuted to a compatible type
    Unsupported(syn::Type), // these are types that `deno_bindgen` currently does not support or have no way to be used in JavaScript space
}

impl IrType {
    pub fn is_trivial(&self) -> bool {
        match &self {
            IrType::Native(_) => true,
            _ => false,
        }
    }
    pub fn is_non_trivial(&self) -> bool {
        match &self {
            IrType::NonTrivial(_) => true,
            _ => false,
        }
    }
}

// used for outputs
pub struct NamedIrType {
    ident: Ident,
    ty: IrType,
}
/*================== TRANSFORM STRUCTS =================*/

pub enum IrItem {
    Fn(IrFn, ItemFn),
    // Struct(IrType, ItemStruct),
    // Impl(IrType, ItemImpl),
    Unknown(TokenStream),
}

pub struct IrFn {
    pub source: ItemFn, // this is the immutable function that will be passed inside the block
    pub inputs: Vec<IrType>,
    pub output: IrType,
}
