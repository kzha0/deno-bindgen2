use core::panic;
use std::process::{Command, Stdio};

use cargo_metadata::MetadataCommand;


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

pub struct Cargo {}

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

    pub fn get_pkg_name(#[cfg(debug_assertions)] cwd: Option<&str>) -> String {
        let mut cmd = MetadataCommand::new();

        #[cfg(debug_assertions)]
        if let Some(cwd) = cwd {
            cmd.current_dir(cwd);
        }

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

        if dylib_target.is_some() {
            root_pkg.name.clone()
        } else {
            panic!(
                "no `cdylib` library target found in package `{}`",
                root_pkg.name
            );
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
            // .arg("--cfg") // pass these commands when building
            // .arg("deno_bindgen")
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // these tests link to the `deno-bindgen2-test` package

    #[test]
    fn test_get_metadata() {
        let pkg_name = Cargo::get_pkg_name(Some("../test"));
        dbg!(pkg_name);
    }

    #[test]
    fn test_expand() {
        let pkg_name = Cargo::get_pkg_name(Some("../test"));
        let content = Cargo::expand(pkg_name.as_str());
        println!("{content}");
    }
}
