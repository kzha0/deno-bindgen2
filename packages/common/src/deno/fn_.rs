use crate::deno::util::*;
use crate::deno::{FfiFunction, FfiType, RustTypeList, TsType, UserDefined};
use crate::rust::{Association, Attribute, Block, ItemFn};

#[derive(Clone, Debug)]
pub struct TsFunction {
    pub method: TsMethod,
}

#[derive(Clone, Debug)]
pub struct TsMethod {
    pub attr:   Attribute,
    pub ident:  Ident,
    pub inputs: Vec<TsType>,
    pub output: TsType,
    pub assoc:  Option<Association>,
    pub block:  Block,
}

impl ItemFn {
    pub fn unwrap(
        self,
        rust_types: &mut RustTypeList,
        user_defined: &mut UserDefined,
    ) -> (FfiFunction, TsMethod) {
        let (ffi_inputs, inputs): (Vec<FfiType>, Vec<TsType>) = self
            .inputs
            .iter()
            .map(|ty| ty.unwrap(rust_types, user_defined))
            .unzip();

        let (ffi_output, output) = self.output.unwrap(rust_types, user_defined);

        let ident = self.ident;
        let ffi_ident = if let Some(self_ty) = &self.block.self_ty {
            format_ident!("__{}__{ident}", self_ty.to_string())
        } else {
            format_ident!("__{ident}")
        };

        (
            FfiFunction {
                ident:        ffi_ident,
                inputs:       ffi_inputs,
                output:       ffi_output,
                non_blocking: self.attr.has_non_blocking(),
            },
            TsMethod {
                attr: self.attr,
                ident,
                inputs,
                output,
                assoc: self.assoc.clone(),
                block: self.block,
            },
        )
    }
}

impl From<TsMethod> for TsFunction {
    fn from(value: TsMethod) -> Self {
        Self { method: value }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: print



impl TsMethod {
    fn transform() {

    }
}

impl ToTokens for TsMethod {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let TsMethod {
            ref attr,
            ref ident,
            ref inputs,
            ref output,
            ref assoc,
            ref block,
        } = self;
        let Block {
            args,
            in_stmts,
            out_stmt,
            self_ty,
        } = block;

        if let Some(assoc) = assoc {
            // handle constructor, destructor etc.
        }


        let mut fn_args = Vec::new();
        let mut call_args = Vec::new();
        for (i, input) in inputs.iter().enumerate() {
            let arg = args.get(i).expect("error: this function's arguments have not been processed. call `transform` on this item");
            call_args.push(quote! { #arg });
            fn_args.push(quote! { #arg: #input });
        }


        tokens.extend(quote! {
            #ident()

        });
    }
}

impl TsFunction {
    fn transform() {

    }
}


impl ToTokens for TsFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let method = &self.method;
        let async_ = if self.method.attr.has_non_blocking() {
            quote! { async }
        } else {
            TokenStream::new()
        };

        tokens.extend(quote! {
            export #async_ function #method
        });
    }
}
