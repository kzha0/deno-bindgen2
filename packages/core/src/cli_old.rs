#![allow(non_snake_case)]

use std::path::PathBuf;

use anyhow::format_err;
use cargo_metadata::Message;
use clap::Parser;
use deno_bindgen2_common::DenoParser;
use dlopen2::wrapper::WrapperApi;

/*------------------------------------------------------------------------*/

type Result<T> = anyhow::Result<T>;

#[derive(Parser, Debug)]
#[command(
    name = "deno-bindgen2",
    version,
    about = "A Rust FFI bindings generator for Deno. Use this CLI tool with the `deno_bindgen2` library crate",
    long_about = None
)]
struct CliOpts {
    #[arg(short, long)]
    /// Build in release mode
    release: bool,

    #[arg(short, long)]
    /// Path to output file
    out: Option<PathBuf>,

    /// Output as a lazy-loaded module
    #[arg(short, long)]
    lazy_init: Option<bool>,

    /// Embed adapter libraries into the final module instead of fetching remotely
    #[arg(short, long)]
    embed: Option<bool>,

    // Supress prompts and use default values for missing arguments
    #[arg(short, long)]
    suppress: Option<bool>
}

/*------------------------------------------------------------------------*/

fn main() -> Result<()> {
    let cli_opts = CliOpts::parse();
    let cwd = &std::env::current_dir().unwrap();

    let pkg_name = cargo_metadata::MetadataCommand::new()
        .exec()
        .or_else(|err| Err(format_err!("failed to execute `cargo metadata`: {:?}", err)))?
        .root_package()
        .ok_or_else(|| format_err!("failed to retrieve root package name"))?
        .name
        .clone();

    // TODO: investigate tradeoffs of separating FFIData from the binaries
    let cdylib_path = cargo_build(cwd, cli_opts.release)?;

    /*---------------------------------------------*/

    #[derive(WrapperApi)]
    struct ParserHook {
        __DENOBINGDEN_CLI_HOOK: unsafe fn(parser: DenoParser) -> Result<()>,
    }

    unsafe {
        let dlopen = dlopen2::wrapper::Container::<ParserHook>::load(cdylib_path.clone())
            .map_err(|err| format_err!("failed to load shared library: {:?}", err))?;

        // arbitrarily execute a function from the dylib. if the dylib was built as intended, this function call should point to the parser
        dlopen.__DENOBINGDEN_CLI_HOOK(DenoParser {
            out_file:  cli_opts.out,
            cdylib:    cdylib_path.display().to_string(),
            lazy_init: cli_opts.lazy_init,
            embed:   cli_opts.embed,
        })?;
    }

    println!("Finished generating bindings for {pkg_name}");

    Ok(())
}

/*------------------------------------------------------------------------*/

fn cargo_build(cwd: &PathBuf, release: bool) -> Result<PathBuf> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.current_dir(cwd)
        .arg("build")
        .arg("--lib")
        .arg("--message-format=json")
        .stdout(std::process::Stdio::piped());

    if release {
        cmd.arg("--release");
    }

    let cmd_out = cmd.output()?;

    if !cmd_out.status.success() {
        Err(format_err!(
            "failed to execute `cargo`: process exited with {}\nfull command: {:?}",
            cmd_out.status,
            cmd
        ))
    } else {
        let cargo_out = std::io::BufReader::new(cmd_out.stdout.as_slice());
        let mut artifact_paths = Vec::new();

        for msg in Message::parse_stream(cargo_out) {
            if let Err(err) = msg {
                return Err(format_err!("failed to parse cargo output: {:?}", err));
            }

            match msg.unwrap() {
                Message::CompilerArtifact(artifact) => {
                    // check to ensure the library is set to type `cdylib`
                    if artifact.target.kind.contains(&"cdylib".to_string()) {
                        let path = artifact.filenames[0].to_string();
                        artifact_paths.push(PathBuf::from(path));
                    }
                },
                _ => (),
            }
        }

        if let Some(path) = artifact_paths.pop() {
            #[cfg(target_os = "windows")]
            let path = path
                .strip_prefix(&cwd)
                .expect("path is not a prefix of cwd");

            Ok(path)
        } else {
            Err(format_err!("failed to retrieve cargo build artifacts\nis your library set to the `cdylib` type?"))
        }
    }
}
