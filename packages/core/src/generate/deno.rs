use std::{
    borrow::Cow, fmt::Display, io::Write, path::Path
};

use ts_quote::ts_string;

use crate::*;

// MARK: ENTRYPOINT GENERATOR

pub struct Generator<'a> {
    items: &'static [RawItem],
    dl_path:  &'a Path,
    lazy:    bool,
}

impl<'a> Generator<'a> {
    pub fn new(
        items: &'static [RawItem],
        dl_path:  &'a Path,
        lazy:    bool,
    ) -> Self {
        Self{
            items,
            dl_path,
            lazy
        }
    }
}

impl<'a> Generator<'a> {
    pub fn generate<W: Write>(Generator{
        items,
        dl_path,
        lazy
    }: Self, mut writer: W) -> std::io::Result<()> {

        //*-------------------------------- FILTER ITEMS ------------------------------*/

        let mut raw_fns: Vec<&'static RawFn> = Vec::new();
        let mut raw_structs: Vec<&'static RawStruct> = Vec::new();

        for item in items.iter() {
            match item {
                RawItem::Fn(raw_fn) => raw_fns.push(raw_fn),
                RawItem::Struct(raw_struct) => raw_structs.push(raw_struct),
            };
        };

        //*-------------------------------- UNPARSE ITEMS ------------------------------*/

        let mut symbols: Vec<SymbolFn> = Vec::new();
        let mut export_fns: Vec<ExportFn<'a>> = Vec::new();
        // let mut export_classes: Vec<ExportClass> = Vec::new();

        for raw_fn in raw_fns {

            let mut export_fn = ExportFn::default();
            export_fn.ident = raw_fn.ident.into();

            let parameters = export_fn.with_inputs(raw_fn.ident, raw_fn.raw_inputs);
            let (result, mut out_fns) = export_fn.with_output(&raw_fn.raw_output);

            symbols.push(SymbolFn{
                ident: raw_fn.ident.into(),
                parameters,
                result,
                non_blocking: raw_fn.non_blocking,
            });

            symbols.append(&mut out_fns);

            export_fns.push(export_fn);
        };

        let dl_open = DlOpen::new(dl_path, symbols, lazy);

        write!(writer, "{}", ts_string! {
            #dl_open
        })?;

        for export_fn in export_fns {
            write!(writer, "{}", ts_string! {
                #export_fn
            })?;
        }

        Ok(())
    }
}

// MARK: UNPARSE INPUTS

impl RawType {
    fn parse_input<'a>(
        &'static self,
        ident_prefix: Cow<'a, str>
    ) -> (PatType<'a>, Block<'a>, Vec<Expr<'a>>, Vec<&'static Trivial>) {

        let in_arg: PatType<'a>;
        let mut in_stmts = Block::new();
        let mut in_exprs: Vec<Expr> = Vec::new();
        let mut ffi_args: Vec<&'static Trivial> = Vec::new();


        // TODO: handle custom types
        // scan for CustomTypes
        // add to a list of identified custom type strings


        match *self {
            RawType::Trivial(ref trivial_ty) => {
                in_arg = PatType::new(ident_prefix.clone(), trivial_ty.into());
                in_exprs.push(ident_prefix.into());

                ffi_args.push(trivial_ty);
            },
            RawType::Paren(raw_ty) => {
                let paren_ident = format!("{ident_prefix}_paren");

                let (
                    this_in_arg,
                    mut this_in_stmts,
                    mut this_in_exprs,
                    mut this_ffi_args
                ) = raw_ty.parse_input(paren_ident.clone().into());

                in_arg = PatType::new(
                    ident_prefix.clone(),
                    TsType::Tuple([this_in_arg.ty].to_vec())
                );

                in_stmts.push(ts_string! {
                    const #paren_ident = #ident_prefix[0];
                }.into());
                in_stmts.append(&mut this_in_stmts);

                in_exprs.append(&mut this_in_exprs);

                ffi_args.append(&mut this_ffi_args);
            },
            RawType::Tuple(raw_tys) => {

                // problem: avoid variable scope pollution
                // solution: anonymous functions to contain variable declarations in each local scope

                // since this is a decomposing function, there will always be one input TsType
                // the input may cascade as a complex type
                // how to elevate scope of a return type with many items from local to the parent scope

                let mut tup_in_args: Vec<TsType> = Vec::new();

                for (this_index, raw_ty) in raw_tys.iter().enumerate() {
                    let tup_ident = format!("{ident_prefix}_tup_{this_index}");
                    let (
                        this_in_arg,
                        mut this_in_stmts,
                        mut this_in_exprs,
                        mut this_ffi_args,
                    ) = raw_ty.parse_input(tup_ident.clone().into());

                    tup_in_args.push(this_in_arg.ty);

                    let tup_ident = this_in_arg.pat;
                    in_stmts.push(ts_string! {
                        const #tup_ident = #ident_prefix[#this_index];
                    }.into());
                    in_stmts.append(&mut this_in_stmts);

                    in_exprs.append(&mut this_in_exprs);
                    ffi_args.append(&mut this_ffi_args);
                };

                in_arg = PatType::new(ident_prefix, TsType::Tuple(tup_in_args))
            },
            RawType::Slice(raw_ty) => {

                let buf_ty = match raw_ty {
                    RawType::Trivial(trivial_ty) =>  {
                        trivial_ty
                    },
                    _ => unreachable!(),
                };

                in_arg = PatType::new(ident_prefix.clone(), buf_ty.into());

                // let buf_ident = format!("{ident_prefix}_buf");
                // in_stmts.push(ts_string! {
                //     const #buf_ident = new TextEncoder().encode(#ident_prefix);
                // }.into());

                let len_ident = ts_string! {#ident_prefix.byteLength};
                in_exprs.append(&mut [ident_prefix.into(), len_ident.into()].to_vec());

                // TODO: fix type coercion of number(u32) to bigint(usize)
                ffi_args.append(&mut [&buf_ty, &Trivial::U32].to_vec());
            },
            RawType::Str => {

                in_arg = PatType::new(ident_prefix.clone(), TsType::String);

                let buf_ident = format!("{ident_prefix}_buf");
                in_stmts.push(ts_string! {
                    const #buf_ident = new TextEncoder().encode(#ident_prefix);
                }.into());

                let len_ident = ts_string! {#buf_ident.byteLength};
                in_exprs.append(&mut [buf_ident.into(), len_ident.into()].to_vec());

                // TODO: fix type coercion of number(u32) to bigint(usize)
                ffi_args.append(&mut [&Trivial::Buffer(&Trivial::U8), &Trivial::U32].to_vec());
            },
            RawType::Custom(_) => {
                todo!()
                // match the custom type string to the list of identified custom types

                // if there is a match, use the custom type as a parameter and access the internal pointer object

                // if no match, create an alias type assigning CustomType = Deno.PointerObject | null;
            },
        };

        (in_arg, in_stmts, in_exprs, ffi_args)
    }

    // MARK: UNPARSE OUTPUT

    fn parse_output<'a> (
        &'static self,
        ident_prefix: Cow<'a, str>
    ) -> (TsType, Block<'a>, Expr<'static>, &'static Trivial, Vec<SymbolFn<'static>>) {
        let out_ty: TsType;
        let mut out_stmts = Block::new();
        let out_expr: Expr<'static>;
        let ffi_result: &'static Trivial;
        let mut out_fns: Vec<SymbolFn<'static>> = Vec::new();

        match *self {
            RawType::Trivial(ref trivial_ty) => {
                out_ty = trivial_ty.into();
                out_expr = ts_string! {
                    #ident_prefix
                }.into();

                ffi_result = trivial_ty;

                (out_ty, out_stmts, out_expr, ffi_result, out_fns)
            },
            RawType::Paren(raw_ty) => {
                let (
                    this_out_ty,
                    mut this_out_stmts,
                    this_out_expr,
                    this_ffi_result,
                    mut this_out_fns
                ) = raw_ty.parse_output(ident_prefix.clone());

                out_ty = TsType::Tuple([this_out_ty].to_vec());

                out_stmts.append(&mut this_out_stmts);
                let paren_ident = format!("{ident_prefix}_paren");
                out_stmts.push(ts_string! {
                    const #paren_ident: #out_ty = [#this_out_expr];
                }.into());

                out_expr = paren_ident.into();

                ffi_result = this_ffi_result;
                out_fns.append(&mut this_out_fns);

                (out_ty, out_stmts, out_expr, ffi_result, out_fns)
            },
            RawType::Tuple(raw_tys) => {
                let mut tup_out_tys: Vec<TsType> = Vec::new();
                let mut tup_exprs: Vec<Expr> = Vec::new();

                let mut tup_out_fns: Vec<SymbolFn> = Vec::new();

                for (this_index, raw_ty) in raw_tys.iter().enumerate() {
                    let tup_ident = format!("{ident_prefix}_{this_index}");

                    let (
                        this_out_ty,
                        mut this_out_stmts,
                        this_out_expr,
                        this_ffi_result,
                        mut this_out_fns
                    ) = raw_ty.parse_output(tup_ident.clone().into());

                    tup_out_tys.push(this_out_ty);

                    out_stmts.push(ts_string! {
                        const #tup_ident = symbols.#tup_ident(#ident_prefix!);
                    }.into());
                    out_stmts.append(&mut this_out_stmts);

                    tup_exprs.push(this_out_expr);

                    tup_out_fns.push(SymbolFn{
                        ident: tup_ident.into(),
                        parameters: [&Trivial::Pointer("")].to_vec(),
                        result: this_ffi_result,
                        non_blocking: false
                    });
                    tup_out_fns.append(&mut this_out_fns);
                };

                out_ty = TsType::Tuple(tup_out_tys);

                // TODO: create utility for Punctuated expressions
                let tup_ident = format!("{ident_prefix}_tup");
                let dealloc_ident = format!("{ident_prefix}_dealloc");
                let tup_exprs = tup_exprs.iter().map(|expr| {
                    format!("{expr}, ")
                }).collect::<String>().trim_end_matches(", ").to_string();
                out_stmts.push(ts_string! {
                    const #tup_ident: #out_ty = [#tup_exprs];

                    symbols.#dealloc_ident(#ident_prefix!);
                }.into());

                out_expr = tup_ident.into();

                ffi_result = &Trivial::Pointer("");

                out_fns.append(&mut tup_out_fns);
                out_fns.push(SymbolFn{
                    ident: dealloc_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: &Trivial::Void,
                    non_blocking: false
                });

                (out_ty, out_stmts, out_expr, ffi_result, out_fns)
            },
            RawType::Slice(raw_ty) => {

                // TODO: deno will always output slices or buffers as Uint8Arrays
                // this byte array must then be converted into the appropriate-sized array

                let buf_ty = match raw_ty {
                    RawType::Trivial(trivial_ty) =>  {
                        trivial_ty
                    },
                    _ => unreachable!(),
                };

                out_ty = buf_ty.into();

                let ptr_ident = format!("{ident_prefix}_ptr");
                let len_ident = format!("{ident_prefix}_len");
                let dealloc_ident = format!("{ident_prefix}_dealloc");

                let buf_ident = format!("{ident_prefix}_buf");
                out_stmts.push(ts_string! {
                    const #ptr_ident = symbols.#ptr_ident(#ident_prefix!) as Deno.PointerObject | null;
                    const #len_ident = symbols.#len_ident(#ident_prefix!);
                    const #buf_ident = new #out_ty(Deno.UnsafePointerView.getArrayBuffer(#ptr_ident!, #len_ident));

                    symbols.#dealloc_ident(#ident_prefix!);
                }.into());

                out_expr = buf_ident.into();

                ffi_result = &Trivial::Pointer("");

                out_fns.push(SymbolFn{
                    ident: ptr_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: buf_ty,
                    non_blocking: false
                });
                out_fns.push(SymbolFn{
                    ident: len_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: &Trivial::U32,
                    non_blocking: false
                });
                out_fns.push(SymbolFn{
                    ident: dealloc_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: &Trivial::Void,
                    non_blocking: false
                });

                (out_ty, out_stmts, out_expr, ffi_result, out_fns)
            },
            RawType::Str => {

                // TODO: use the slice type unparser to reduce code clutter

                out_ty = TsType::String;

                let ptr_ident = format!("{ident_prefix}_ptr");
                let len_ident = format!("{ident_prefix}_len");
                let dealloc_ident = format!("{ident_prefix}_dealloc");

                let buf_ident = format!("{ident_prefix}_buf");
                let str_ident = format!("{ident_prefix}_string");

                out_stmts.push(ts_string! {
                    const #ptr_ident = symbols.#ptr_ident(#ident_prefix!) as Deno.PointerObject | null;
                    const #len_ident = symbols.#len_ident(#ident_prefix!);
                    const #buf_ident = new Uint8Array(Deno.UnsafePointerView.getArrayBuffer(#ptr_ident!, #len_ident));
                    const #str_ident = new TextDecoder("utf-8").decode(#buf_ident);

                    symbols.#dealloc_ident(#ident_prefix!);
                }.into());

                out_expr = str_ident.into();

                ffi_result = &Trivial::Pointer("(*mut u8, usize)");

                out_fns.push(SymbolFn{
                    ident: ptr_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: &Trivial::Buffer(&Trivial::U8),
                    non_blocking: false
                });
                out_fns.push(SymbolFn{
                    ident: len_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: &Trivial::U32,
                    non_blocking: false
                });
                out_fns.push(SymbolFn{
                    ident: dealloc_ident.into(),
                    parameters: [ffi_result].to_vec(),
                    result: &Trivial::Void,
                    non_blocking: false
                });

                (out_ty, out_stmts, out_expr, ffi_result, out_fns)
            },
            RawType::Custom(_) => todo!(),
        }
    }
}

// MARK: CONSTRUCTORS

// creates a symbol object of format:

// #ident: {
//     parameters: [
//         "buffer",
//         "usize"
//     ],
//     result: "void"
// },

// MARK: SYMBOLS

struct SymbolFn<'a> {
    ident: Cow<'a, str>,
    parameters: Vec<&'static Trivial>,
    result: &'static Trivial,
    non_blocking: bool,
}

impl<'a> Display for SymbolFn<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut internal = if self.parameters.is_empty() {
            ts_string! {
                parameters: [],
            }
        } else {
            let mut inputs = String::new();
            for param in &self.parameters {
                let param = param.to_ffi_type();
                inputs.push_str(&ts_string! {
                    #param,
                });
            }

            ts_string! {
                parameters: [
                    #inputs
                ],
            }
        };

        let result = self.result.to_ffi_type();
        internal.push_str(&ts_string! {
            result: #result,
        });

        if self.non_blocking {
            internal.push_str(&ts_string! {
                non_blocking: true
            });
        };

        let ident = self.ident.clone();
        write!(f, "{}", ts_string! {
            #ident: {
                #internal
            },
        })
    }
}

// MARK: DLOPEN

struct DlOpen<'a> {
    dl_path: &'a Path,
    symbols: Vec<SymbolFn<'a>>,
    lazy: bool,
}

impl<'a> DlOpen<'a> {
    fn new(
        dl_path: &'a Path,
        symbols: Vec<SymbolFn<'a>>,
        lazy: bool
    ) -> Self {
        Self{
            dl_path,
            symbols,
            lazy,
        }
    }
}

impl<'a> Display for DlOpen<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut internal = String::new();
        for symbol in &self.symbols {
            internal.push_str(&ts_string! {
                #symbol
            });
        };

        let path = format!("\"{}\"", self.dl_path.display());
        let out = if self.lazy {
            ts_string! {
                let symbols: any;

                export function load(path: string = #path {
                    const {{ dlopen }} = Deno;
                    const {{ symbols: symbols_ }} = dlopen(path, {
                        #internal
                    });
                });
            }
        } else {
            ts_string! {
                const { symbols } = Deno.dlopen(#path, {
                    #internal
                });
            }
        };

        write!(f, "{}", out)
    }
}

// MARK: EXPORT FNS

#[derive(Debug, Clone, Default)]
struct ExportFn<'a> {
    ident: Cow<'a, str>,
    parameters: Vec<PatType<'a>>,
    result: TsType,
    stmts: Block<'a>,
}

impl<'a> ExportFn<'a> {
    fn with_inputs(&mut self, fn_ident: &'a str, raw_inputs: &'static [RawType]) -> Vec<&'static Trivial> {

        let mut in_args: Vec<PatType<'a>> = Vec::new();
        let mut in_stmts: Block = Block::new();
        let mut in_exprs: Vec<Expr> = Vec::new();
        let mut ffi_args: Vec<&'static Trivial> = Vec::new();

        for raw_input in raw_inputs {
            let (
                this_in_arg,
                mut this_in_stmts,
                mut this_in_exprs,
                mut this_ffi_args,
            ) = raw_input.parse_input(format!("arg_{}", in_args.len()).into());

            in_args.push(this_in_arg);
            in_stmts.append(&mut this_in_stmts);
            in_exprs.append(&mut this_in_exprs);
            ffi_args.append(&mut this_ffi_args);
        };

        let call_expr: String = in_exprs.iter().map(|expr| {
            format!("{expr}, ")
        }).collect();
        let call_expr = call_expr.trim_end_matches(", ");

        self.ident = fn_ident.into();
        let ident = fn_ident;
        in_stmts.push(ts_string! {
            symbols.#ident(#call_expr);
        }.into());

        self.parameters = in_args;
        self.stmts.append(&mut in_stmts);


        ffi_args
    }

    fn with_output(&mut self, raw_output: &'static RawType) -> (&'static Trivial, Vec<SymbolFn<'a>>) {

        if *raw_output == RawType::Trivial(Trivial::Void) {
            self.result = TsType::Void;

            match raw_output {
                RawType::Trivial(trivial_ty) => {
                    (trivial_ty, Vec::new())
                },
                _ => unreachable!()
            }
        } else {
            let fn_ident = self.ident.clone();

            let (
                out_ty,
                this_out_stmts,
                out_expr,
                ffi_result,
                out_fns
            ) = raw_output.parse_output(fn_ident.clone());

            self.result = out_ty;

            let call_expr = self.stmts.swap_remove(self.stmts.len() - 1).to_string().trim_end_matches("; ").to_string();

            let call_res: TsType = ffi_result.into();
            self.stmts.push(ts_string! {
                const #fn_ident = #call_expr as #call_res;
                #this_out_stmts
                return #out_expr;
            }.into());

           (ffi_result, out_fns)
        }
    }
}

impl<'a> Display for ExportFn<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parameters = if self.parameters.is_empty() {
            String::new()
        } else {
            let buf: String = self.parameters.iter().map(|param| {
                format!("{param}, ")
            }).collect();
            buf.trim_end_matches(", ").to_string()
        };

        let ident = self.ident.clone().trim_start_matches("_").to_string();
        let result = &self.result;
        let stmts = &self.stmts;
        write!(f, "{}", ts_string! {
            export function #ident(#parameters): #result {
                #stmts
            }
        })
    }
}

// MARK: UTILITY TYPES

#[derive(Clone, Debug, Default)]
pub enum TsType {
    #[default]
    Void,
    Bool,
    Number,
    BigInt,
    Pointer,
    TypedArray(TypedArray),
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    String,
    Object(&'static str),
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

impl Display for TsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsType::Void => write!(f, "void"),
            TsType::Bool => write!(f, "boolean"),
            TsType::Number => write!(f, "number"),
            TsType::BigInt => write!(f, "bigint"),
            TsType::Pointer => write!(f, "Deno.PointerObject | null"),
            TsType::TypedArray(ty_array) => {
                match ty_array {
                    TypedArray::Uint8Array   => write!(f, "Uint8Array"),
                    TypedArray::Uint16Array  => write!(f, "Uint16Array"),
                    TypedArray::Uint32Array  => write!(f, "Uint32Array"),
                    TypedArray::Uint64Array  => write!(f, "Uint64Array"),
                    TypedArray::Int8Array    => write!(f, "Int8Array"),
                    TypedArray::Int16Array   => write!(f, "Int16Array"),
                    TypedArray::Int32Array   => write!(f, "Int32Array"),
                    TypedArray::Int64Array   => write!(f, "Int64Array"),
                    TypedArray::Float32Array => write!(f, "Float32Array"),
                    TypedArray::Float64Array => write!(f, "Float64Array"),
                }
            },
            TsType::Array(ts_ty) => write!(f, "{ts_ty}[]"),
            TsType::Tuple(ts_tys) => {
                let ts_tys: String = ts_tys.iter().map(|ts_ty| {
                    format!("{ts_ty}, ")
                }).collect();
                let ts_tys = ts_tys.trim_end_matches(", ");

                write!(f, "[{ts_tys}]")
            },
            TsType::String => write!(f, "string"),
            // TODO: Resolve how custom types should be written, either as a class or pointer
            TsType::Object(ts_ty) => write!(f, "{ts_ty}"),
        }
    }
}

impl From<&'static Trivial> for TsType {
    fn from(value: &'static Trivial) -> Self {
        match value {
            Trivial::Void  => TsType::Void,
            Trivial::Bool  => TsType::Bool,
            Trivial::U8
            | Trivial::U16
            | Trivial::U32
            | Trivial::I8
            | Trivial::I16
            | Trivial::I32
            | Trivial::F32
            | Trivial::F64   => TsType::Number,
            Trivial::U64
            | Trivial::I64
            | Trivial::Usize
            | Trivial::Isize => TsType::BigInt,
            Trivial::Pointer(_) => TsType::Pointer,
            Trivial::Buffer(ty) => {
                match ty {
                    Trivial::U8    => TsType::TypedArray(TypedArray::Uint8Array),
                    Trivial::U16   => TsType::TypedArray(TypedArray::Uint16Array),
                    Trivial::U32   => TsType::TypedArray(TypedArray::Uint32Array),
                    Trivial::U64
                    | Trivial::Usize => TsType::TypedArray(TypedArray::Uint64Array),

                    // TODO: refactor code and mitigate coercion of usize to u64
                    // note: there is an arbitrary assumption made here about platform word sizes to be 64 bits wide.
                    // this assumption holds true as of writing, for the following reasons:
                        // deno only works for 64-bit architecture platforms and currently does not support 32-bit machines as of writing
                        // deno's official position on 32-bit support is that they will not be providing support for 32-bit. read here https://github.com/denoland/deno/issues/2295#issuecomment-2329248010
                        // 32-bit support would impose additional development burden and may complicate the code base
                        // also the computing industry has, for the most part, already transitioned to a predominantly 64-bit architecture computing platform

                    // moving forward, this assertion should be refactored so it does not depend upon this word size assumption. one way would be to force the user to explicity annotate a size type into a specific size, or to have a platform word size coercion mechanism that provides support for something as a `IsizeArray/UsizeArray` and convert them to their appropriate typedarray size for the corresponding platform architecture

                    Trivial::I8    => TsType::TypedArray(TypedArray::Int8Array),
                    Trivial::I16   => TsType::TypedArray(TypedArray::Int16Array),
                    Trivial::I32   => TsType::TypedArray(TypedArray::Int32Array),
                    Trivial::I64
                    | Trivial::Isize => TsType::TypedArray(TypedArray::Int64Array),

                    Trivial::F32   => TsType::TypedArray(TypedArray::Float32Array),
                    Trivial::F64   => TsType::TypedArray(TypedArray::Float64Array),
                    _ => unreachable!(),

                    // in the future, Deno's FFI API for buffer types may change. in such case, this unreachable assertion may fail and panic
                }
            },
        }
    }
}

impl Trivial {
    fn to_ffi_type(&self) -> &str {
        match self {
            Trivial::Void  => "\"void\"",
            Trivial::Bool  => "\"bool\"",
            Trivial::U8    => "\"u8\"",
            Trivial::U16   => "\"u16\"",
            Trivial::U32   => "\"u32\"",
            Trivial::U64   => "\"u64\"",
            Trivial::I8    => "\"i8\"",
            Trivial::I16   => "\"i16\"",
            Trivial::I32   => "\"i32\"",
            Trivial::I64   => "\"i64\"",
            Trivial::Usize => "\"usize\"",
            Trivial::Isize => "\"isize\"",
            Trivial::F32   => "\"f32\"",
            Trivial::F64   => "\"f64\"",
            Trivial::Pointer(_) => "\"pointer\"",
            Trivial::Buffer(_) => "\"buffer\""
        }
    }
}

// MARK: TOKENIZER STRUCTS

#[derive(Debug, Clone, Default)]
struct PatType<'a> {
    pat: Cow<'a, str>,
    ty: TsType
}
impl<'a> PatType<'a> {
    fn new<T: Display>(pat: T, ty: TsType) -> Self
    where Cow<'a, str>: From<T>
    {
        Self { pat: pat.into(), ty }
    }
}
impl<'a> Display for PatType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.pat, self.ty)
    }
}

#[derive(Debug, Clone, Default)]
struct Expr<'a> (Cow<'a, str>);
impl<'a, T: Display> From<T> for Expr<'a> where Cow<'a, str>: From<T> {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
impl<'a> Display for Expr<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
struct Stmt<'a> (Cow<'a, str>);
impl<'a, T: Display> From<T> for Stmt<'a> where Cow<'a, str>: From<T> {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
impl<'a> Display for Stmt<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Default)]
struct Block<'a> (Vec<Stmt<'a>>);
impl<'a> Block<'a> {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn push(&mut self, value: Stmt<'a>) {
        self.0.push(value);
    }
    fn append(&mut self, value: &mut Block<'a>) {
        self.0.append(&mut value.0);
    }
    fn swap_remove(&mut self, index: usize) -> Stmt<'a> {
        self.0.swap_remove(index)
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> Display for Block<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for stmt in &self.0 {
            write!(f, "{}", stmt)?;
        }
        Ok(())
    }
}
