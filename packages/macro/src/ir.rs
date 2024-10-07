
#[allow(unused)]
#[derive(Clone, Debug)]
pub enum RawItem {
    Fn(RawFn),
    Struct(RawStruct),
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct RawStruct {
    pub ident:   &'static str,
    pub methods: &'static [RawFn],
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct RawFn {
    pub ident:        &'static str,
    pub raw_inputs:   &'static [RawType],
    pub raw_output:   RawType,
    pub non_blocking: bool,
    pub _internal:    bool,
    pub _constructor: bool,
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum RawType {
    Trivial(Trivial),
    Paren(&'static RawType),
    Tuple(&'static [RawType]),
    Slice(&'static RawType),  // [pointer, usize]
    Str,                  // [pointer, usize]
    Custom(&'static str), // [pointer]
}

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
    Pointer(&'static str),
    Buffer(Box<Trivial>), // in the future, Deno's FFI API for buffer types may change
    // Function, // TODO: unimpeleted types that are supported by the ffi
    // Struct(Box<[Trivial]>),
}

