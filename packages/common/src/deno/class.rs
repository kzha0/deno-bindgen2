use std::collections::BTreeMap;

use crate::deno::util::*;
use crate::deno::{FfiFunction, FfiType, TsMethod, TsModule};
use crate::rust::ItemImpl;

#[derive(Clone, Debug)]
pub struct TsClass {
    pub methods: Vec<TsMethod>,
}

#[derive(Clone, Debug, Default)]
pub struct ClassDefs {
    pub store: BTreeMap<Ident, TsClass>,
}

impl ItemImpl {
    pub fn unwrap(self, module: &mut TsModule) {
        let mut methods = Vec::new();
        for item in self.items {
            let method = item.unwrap(module);
            methods.push(method);
        }

        if let Some(ts_class) = module.class_defs.store.get_mut(&self.self_ty) {
            ts_class.methods.append(&mut methods);
        } else {
            module.ffi_lib.interface.push_fn(FfiFunction {
                ident:        format_ident!("__{}__drop", &self.self_ty.to_string()),
                inputs:       vec![FfiType::Pointer],
                output:       FfiType::Void,
                non_blocking: false,
            });
            module
                .class_defs
                .store
                .insert(self.self_ty, TsClass { methods });
        }
    }
}

impl ToTokens for ClassDefs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for (ident, TsClass { methods }) in &self.store {
            let mut _methods = Vec::new();
            for method in methods {
                let method = method.print();
                _methods.push(method);
            }

            tokens.extend(quote! {
                export class #ident extends RustPrototype<#ident> {
                    #(#_methods)*
                }
            });
        }
    }
}

/* -------------------------------------------------------------------------- */

// MARK: tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deno::{TsFormat, TsModule};
    use crate::{parse_quote, prettify};

    macro_rules! test_transform {
        ($($tt:tt)*) => {
            println!("[rust source]\n{}", prettify!(stringify!($($tt)*)));

            let source = parse_quote!(ItemImpl, $($tt)*);

            let mut export = source.clone();
            export.transform();
            println!("[rust wrapper]\n{}", prettify!(export.to_token_stream().to_string().as_str()));

            let mut module = TsModule::default();
            source.unwrap(&mut module);

            let ffi_lib = TsFormat::format(module.ffi_lib.to_token_stream().to_string());
            println!("[ts ffi]\n{}", ffi_lib);

            let class_defs = TsFormat::format(module.class_defs.to_token_stream().to_string());
            println!("[ts mod]\n{}", class_defs);
        };
    }


    #[test]
    #[cfg(feature = "cli")]
    fn test_print() {
        test_transform!(
            impl CustomType {
                fn test_fn(&self) {}
            }
        );
    }
}
