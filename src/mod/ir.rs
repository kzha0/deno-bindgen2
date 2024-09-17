use syn::{Ident, Type};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum TrivialType {
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
    Pointer(Type),
    Buffer,
    // Function, // TODO: unimpeleted types that are supported by the ffi

    // Struct(Box<[Trivial]>),
}

#[derive(Clone, Debug)]
pub struct ParameterType {
    pub ty: TrivialType,
    pub ident: Ident,
}

/*================================ COMPOSITE TYPES ==============================*/

#[derive(Clone, Debug)]
pub struct ParenType {
    pub elem: Box<IrType>
}

#[derive(Clone, Debug)]
pub struct TupleType {
    pub elems: Vec<IrType>,
}

#[derive(Clone, Debug)]
pub struct ReferenceType {
    pub _mut: bool,
    pub elem: Box<IrType>,
}

#[derive(Clone, Debug)]
pub struct SliceType {
    pub ptr: ParameterType,
    pub len: ParameterType,
}


// In Deno, vector types may be implemented as an array of a single type. If the type is unsupported, it could contain a pointer object to a custom type
#[derive(Clone, Debug)]
pub struct VecType {

}

#[derive(Clone, Debug)]
pub struct CustomType {
    pub ty: Ident,
    pub ptr: ParameterType,
}

// Provide lifetime annotations to self types in JavaScript to check validity
#[derive(Clone, Debug)]
pub struct SelfType {

}

#[derive(Clone, Debug)]
pub enum IrType {
    // Terminating type. Appears directly in function arguments
    Parameter(ParameterType),

    // Recursive types. Must be composed/decomposed
    Paren(ParenType),
    Tuple(TupleType),

    // Container types
    Reference(ReferenceType),
    Slice(SliceType),
    Str(SliceType),
    String(SliceType),

    Custom(CustomType), // avoid creating double references
}
/*================================ PARAMETER TYPES ==============================*/

// structs for generating a collection of associated functions based on whether the result type is supported or not
pub struct TrivialFn {
    pub inputs: Vec<IrType>,
    pub output: TrivialType,
}

pub enum CompositeFn {
    Tuple,  // returns a tuple type. construct tuple from each element. provide a deallocator
    Custom, // returns a custom type or a custom fn. should provide a deallocator
}


pub enum SymbolSignature {
    Trivial(TrivialFn),
    Composite()
}

