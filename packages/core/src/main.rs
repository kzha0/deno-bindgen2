use std::{
    os::unix::process::CommandExt, path::{
        Path,
        PathBuf,
    }, process::{
        Command,
        Stdio,
    }
};

use clap::Parser;
use dlopen2::wrapper::{
    Container,
    WrapperApi,
};

pub struct Artifact {
    pub path:          PathBuf,
    pub manifest_path: PathBuf,
}

#[derive(Parser, Debug)]
#[structopt(name = "deno_bindgen2_cli", about = "Generate rust bindings for Deno in TypeScript. Use this CLI tool with the `deno_bindgen2` library crate")]
struct Opt {
    #[structopt(short, long)]
    /// Build in release mode
    release: bool,

    #[structopt(short, long)]
    /// Path to output file
    out: Option<PathBuf>,

    #[structopt(short, long)]
    lazy_init: bool,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::parse();

    //*-------------------------------- CARGO BUILD ------------------------------*/

    let cwd = std::env::current_dir().unwrap();

    let mut cmd = Command::new("cargo");
    cmd.current_dir(cwd)
        .arg("build")
        .arg("--lib")
        .arg("--message-format=json")
        .stdout(Stdio::piped());

    if opt.release {
        cmd.arg("--release");
    };

    let output = cmd.output()?;


    let path = if output.status.success() {
        let reader = std::io::BufReader::new(output.stdout.as_slice());
        let mut artifacts = Vec::new();

        for msg in cargo_metadata::Message::parse_stream(reader) {
            match msg.unwrap() {
                cargo_metadata::Message::CompilerArtifact(artifact) => {
                    if artifact.target.kind.contains(&"cdylib".to_string()) {
                        artifacts.push(Artifact {
                            path:          PathBuf::from(artifact.filenames[0].to_string()),
                            manifest_path: PathBuf::from(artifact.manifest_path.to_string()),
                        });
                    }
                },
                _ => {},
            }
        }

        if let Some(artifact) = artifacts.pop() {
            artifact.path
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to parse cargo output",
            ))?
        }
    } else {
        println!(
            "failed to execute `cargo`: exited with {}\n  full command: {:?}",
            output.status, cmd
        );

        std::process::exit(1);
    };
    #[cfg(target_os = "windows")]
    let path = path
        .strip_prefix(&cwd)
        .expect("path is not a prefix of cwd");

    let name = {
        let metadata = cargo_metadata::MetadataCommand::new()
            .exec()
            .map_err(|e| {
                println!("failed to execute `cargo metadata`: {}", e);
                std::process::exit(1);
            })
            .unwrap();

        metadata.root_package().unwrap().name.clone()
    };

    //*-------------------------------- DLOPEN ------------------------------*/
    unsafe { dlopen(&path, opt.out, opt.lazy_init)? };

    println!("Ready {name}");

    Ok(())
}

#[derive(WrapperApi)]
struct Api {
    __deno_bindgen2_init: unsafe fn(opt: deno_bindgen2::Options),
}

unsafe fn dlopen(path: &Path, out: Option<PathBuf>, lazy_init: bool) -> std::io::Result<()> {
    let cont: Container<Api> = Container::load(path).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to load library: {}", err),
        )
    })?;

    cont.__deno_bindgen2_init(deno_bindgen2::Options {
        target: deno_bindgen2::Target::Deno,
        out,
        local_dylib_path: path.to_path_buf(),
        lazy_init,
    });

    Ok(())
}
