use dprint_plugin_typescript::configuration::ConfigurationBuilder;
use dprint_plugin_typescript::format_parsed_source;
pub use proc_macro2::{Ident, TokenStream};
pub use quote::{format_ident, quote, ToTokens};
pub use syn::parse::Parse;
pub use syn::{braced, bracketed, LitStr, Token};

pub struct TsFormat;

impl TsFormat {
    pub fn format(source: String) -> String {
        let module = deno_ast::parse_module(deno_ast::ParseParams {
            specifier:      deno_ast::ModuleSpecifier::parse("file://")
                .expect("failed to parse ts source"),
            media_type:     deno_ast::MediaType::TypeScript,
            text:           deno_ast::SourceTextInfo::from_string(source).text(),
            capture_tokens: true,
            scope_analysis: false,
            maybe_syntax:   None,
        })
        .map_err(|err| {
            eprintln!("failed to parse TypeScript code: {:#?}", err);
            panic!()
        })
        .unwrap();

        format_parsed_source(
            &module,
            &ConfigurationBuilder::new()
                .indent_width(4)
                .line_width(100)
                .build(),
        )
        .expect("failed to format TypeScript output")
        .unwrap_or(String::new())
    }
}
