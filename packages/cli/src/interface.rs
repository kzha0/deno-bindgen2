use std::path::PathBuf;

use clap::{Args, Parser};

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
    /// Path to output file. Outputs to console if unspecified
    #[arg(short = 'o', long, value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Build for release
    #[arg(short = 'r', long)]
    pub release: bool,

    #[command(flatten)]
    pub codegen_flags: Option<CodegenFlags>,

    // Operation mode flags
    #[command(flatten)]
    pub op_mode: Option<OpMode>,
}

#[derive(Clone, Debug, Args)]
pub struct OpMode {
    /// Disables source code expansion and module scanning
    #[arg(short = 'e', long)]
    no_expand: bool,

    /// Disables guided mode
    #[arg(short = 'i', long)]
    no_interactive: bool,

    /// Disables generation/linking of utility modules
    #[arg(short = 'm', long)]
    no_modules: bool,
}

#[derive(Clone, Debug, Args)]
pub struct CodegenFlags {
    /// Lazily load the generated module. Useful when the dylib will be
    /// fetched/downloaded remotely.
    #[arg(short = 'l', long)]
    lazy_load: bool,

    /// Import utility modules remotely in generated module. Incompatible with
    /// `--link-embed`
    #[arg(short = 'R', long, group = "link")]
    link_remote: bool,

    /// Embed utility modules in generated module. Incompatible with
    /// `--link-remote`
    #[arg(short = 'E', long, group = "link")]
    link_embed: bool,
}

impl Cli {
    /// attempt to start guided prompt mode for the CLI
    pub fn try_interactive(&mut self) -> bool {
        if let Some(op_mode) = &self.op_mode {
            // if the user did not supress interactive mode
            if op_mode.no_interactive {
                false;
            }
        };
        true
    }

    pub fn with_output(&mut self, output: &str) {
        self.output = Some(PathBuf::from(output))
    }
}
