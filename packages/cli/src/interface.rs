use std::path::PathBuf;

use clap::Parser;
use deno_bindgen2_common::CodegenOpts;


// interactive mode if no subcommand was passed

// parse-expand source file, autodetect modules, prompt for features with
// auto-detected suggestions no parse-expand, prompt for features manually
// no parse-expand, no prompt, include all features
// no parse-expand, no prompt. no features

// default behavior

#[derive(Debug, Parser)]
#[command(
    name = "deno-bindgen2",
    version,
    about = "A Rust FFI bindings generator for Deno. Use this CLI tool with the `deno_bindgen2` library crate",
    long_about = None
)]
pub struct Cli {
    /// Path to output file. Defaults to `<pkg_root>/dist/mod.ts` if unspecified
    #[arg(short = 'o', long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Build for release
    #[arg(short = 'r', long)]
    pub release: bool,

    /// Lazily load the generated module. Useful when the dylib will be
    /// fetched/downloaded remotely.
    #[arg(short = 'l', long)]
    lazy: bool,

    /// Whether to print the rust type declarations in the same typescript
    /// module or to write it on a separate file. Useful when linking together
    /// multiple ffi libraries to use the same rust types
    #[arg(short = 'i', long, group = "link")]
    pub inline: bool,

    /// If false, uses the opaque representations of rust types with no
    /// implementation to interface with rust data structures
    ///
    /// If true, uses the extended rust types with methods for interfacing
    /// with rust data structures, but embeds the ffi symbols on the same dyli
    #[arg(short = 'e', long, default_value_t = true)]
    pub extended: bool,

    /// If provided, writes the extended rust types on a  separate
    /// file and uses the dylib from this path for the typescript representation
    /// of the extended rust types. Incompatible with `inline=true`
    #[arg(short = 'm', long, group = "link")]
    embedded: Option<PathBuf>,

    /// Disables source code expansion and module scanning
    #[arg(short = 'n', long)]
    no_expand: bool,

    /// Set to false to disable guided mode
    #[arg(short = 'I', long)]
    interactive: bool,

    /// Disables generation/linking of utility modules
    #[arg(short = 'N', long)]
    no_modules: bool,
}

impl Cli {
    pub fn to_codegen_opts(&self, file_name: String, dylib_path: PathBuf) -> CodegenOpts {
        CodegenOpts {
            file_name,
            dylib_path: dylib_path
                .to_str()
                .expect("unknown utf8 character on dylib path")
                .to_string(),
            lazy: self.lazy,
            extended: self.extended,
            embedded: self.embedded.clone(),
        }
    }
}
