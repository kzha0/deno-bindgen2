use core::panic;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use cargo_metadata::{Message, MetadataCommand};


/// LIMITATIONS
///
/// the `rustc` macro expansion cannot expand invocations of other external
/// crates, which may be undesired if the user is building a package as part of
/// a project workspace
///
/// this tool only detects modules used within the current package inferred by
/// cargo/rustc. It does not detect upstream crates.

// TODO PERFORMANCE: link directly to rust compiler libraries
// propose api to use native rust AST for procedural macros and `rustc expand`
// for macro expansion result. also propose possible custom preprocessing step
// before compilation

#[derive(Clone, Debug)]
pub struct MetaData {
    pub pkg_name:      String,
    pub lib_name:      String,
    pub pkg_path:      PathBuf,
    pub workspace_dir: PathBuf,
}

impl MetaData {
    pub fn strip_workspace_path(&self, path: &PathBuf) -> PathBuf {
        path.strip_prefix(&self.workspace_dir)
            .expect("pkg path is not a prefix of workspace root path")
            .to_path_buf()
    }
}

pub struct Cargo;

impl Cargo {
    pub fn precheck() {
        // check if:
        // - rustup, cargo and rustc are installed (is this necessary?)
        // - project is configured to use nightly toolchain

        // `whereis {rustup rustc cargo}`
        // trim separated (" ") /usr/bin/cargo /home/.../.cargo/bin/rustup

        const PREFERRED_TOOLCHAIN: &'static str = "nightly";

        let toolchain = Command::new("rustup")
            .arg("show")
            .arg("active-toolchain")
            .output()
            .expect("failed to retrieve rustup toolchain")
            .stdout;
        let toolchain =
            String::from_utf8(toolchain).expect("failed to retrieve active rust toolchain");
        let toolchain = toolchain[..toolchain.find("-").unwrap()].to_string();

        if PREFERRED_TOOLCHAIN != toolchain.as_str() {
            eprintln!("warning: this package is using toolchain `{toolchain}`. it is recommended to use `{PREFERRED_TOOLCHAIN}`");
        }
    }

    pub fn get_metadata() -> MetaData {
        let cmd = MetadataCommand::new();

        let metadata = cmd.exec().expect("failed to execute `cargo metadata`");
        let root_pkg = metadata
            .root_package()
            .expect("failed to retrieve root package name");

        let dylib_target = root_pkg.targets.iter().find(|target| {
            if let Some(kind) = target.kind.first() {
                if kind.as_str() == "cdylib" {
                    if let Some(crate_type) = target.crate_types.first() {
                        if crate_type.as_str() == "cdylib" {
                            return true;
                        }
                    }
                }
            }
            false
        });

        let pkg_name;
        let lib_name;

        if let Some(dylib_target) = dylib_target {
            pkg_name = root_pkg.name.clone();
            lib_name = dylib_target.name.clone();
        } else {
            panic!(
                "no `cdylib` library target found in package `{}`",
                root_pkg.name
            );
        }

        let mut pkg_path = PathBuf::from(root_pkg.manifest_path.clone());
        pkg_path.pop();

        MetaData {
            pkg_name,
            lib_name,
            pkg_path,
            workspace_dir: PathBuf::from(metadata.workspace_root.clone()),
        }
    }

    // run unprety=expanded on the source file
    pub fn expand(pkg_name: &str) -> String {
        let mut cmd = Command::new("cargo");
        cmd.arg("rustc")
            .arg("--package")
            .arg(pkg_name)
            .arg("--lib")
            .arg("--")
            .stderr(Stdio::inherit());


        const UNSTABLE_FLAGS: &[&str] = &["unstable-options", "unpretty=expanded", "no-codegen"];
        for unstable_flag in UNSTABLE_FLAGS {
            cmd.arg(format!("-Z{}", unstable_flag));
        }

        let output = cmd.output().expect("failed to start `cargo rustc` process");

        // [!TODO] panic if an error was emitted

        let content =
            String::from_utf8(output.stdout).expect("failed to parse `rustc` expansion output");
        content
    }

    pub fn build(pkg_name: &str, release: bool, mut cfgs: Vec<&str>) -> PathBuf {
        let mut cmd = Command::new("cargo");
        cmd.arg("+nightly")
            .arg("build")
            .arg("--package")
            .arg(pkg_name)
            .arg("--lib")
            .arg("--message-format=json")
            .stderr(Stdio::inherit())
            .stdout(Stdio::piped());

        cfgs.push("deno_bindgen");
        let cfgs = cfgs.iter().map(|cfg| {
            format!("--cfg {} ", cfg)
        }).collect::<String>();

        cmd.env("RUSTFLAGS", cfgs);

        if release {
            cmd.arg("--release");
        }

        let output = cmd.output().expect("failed to start `cargo build` process");

        let dylib_path;

        if !output.status.success() {
            panic!(
                "failed to execute `cargo`: process exited with {}\nfull command: {:?}",
                output.status, cmd
            )
        } else {
            let cargo_out = std::io::BufReader::new(output.stdout.as_slice());
            let mut artifact_paths = Vec::new();

            for msg in Message::parse_stream(cargo_out) {
                if let Err(err) = msg {
                    panic!("failed to parse cargo output: {:?}", err);
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
                let dylib_path = path
                    .strip_prefix(&cwd)
                    .expect("path is not a prefix of cwd");

                dylib_path = path;
            } else {
                panic!("failed to retrieve cargo build artifacts\nis your library set to the `cdylib` type?")
            }
        }

        dylib_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // these tests link to the `deno-bindgen2-test` package

    #[test]
    fn test_get_metadata() {
        let metadata = Cargo::get_metadata();
        dbg!(&metadata);
    }

    #[test]
    fn test_expand() {
        let metadata = Cargo::get_metadata();
        let content = Cargo::expand(metadata.pkg_name.as_str());
        println!("{content}");
    }

    #[test]
    fn test_build() {
        let metadata = Cargo::get_metadata();
        // let dylib_path = Cargo::build(pkg_name, pkg_rel_path, release)

        let dylib_path = Cargo::build(&metadata.pkg_name, false, vec![]);
        dbg!(&dylib_path);
        /*
                successes:
        ---- cargo::tests::test_build stdout ----
        [packages/cli/src/cargo.rs:236:9] &dylib_path = "/home/rico/Desktop/deno-bindgen2/target/debug/libdeno_bindgen2_test.so"
         */
    }
}
