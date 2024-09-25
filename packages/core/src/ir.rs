use syn::Type;

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
    Pointer(Type),
    Buffer,
    // Function, // TODO: unimpeleted types that are supported by the ffi

    // Struct(Box<[Trivial]>),
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

