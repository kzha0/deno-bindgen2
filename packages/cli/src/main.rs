use std::io::Write;

use deno_bindgen2_common::{File, TsModule};

mod cargo;
mod interface;

use cargo::Cargo;

pub type Result<T> = std::result::Result<T, ()>;

// planned module layout
// main - entrypoint
// cargo - cargo commands
// codegen - code generation? (should'nt this be moved to /common?)
// interface - CLI options and prompt


// guided prompt mode
// parses source code to check modules
/*
fn interactive(args: &mut interface::Cli) -> Result<()> {
    let output = inquire::Text::new(
        "Where should the output file be emitted? (leave blank to output to
console)",     )
    .prompt_skippable()?;

    if let Some(output) = output {
        if output.chars().count() > 0 {
            args.with_output(output.as_str());
        }
    }

    // this command should be invocable in autogenerate mode
    // without needing to prompt users which modules to include
    let source_file = cargo::cargo_expand()?;

    parse::parse_source(source_file)?;


    // [!todo] prelude injection
    // use `rustc --extern `
    // handle insertion of prelude module containing utility module symbols
    // https://doc.rust-lang.org/reference/names/preludes.html#extern-prelude

    Ok(())
}
*/

fn main() -> Result<()> {
    let args: interface::Cli = clap::Parser::parse();

    // if args.try_interactive() {
    //     interactive(&mut args)?;
    // }

    Cargo::precheck();
    let metadata = Cargo::get_metadata();

    let file = Cargo::expand(metadata.pkg_name.as_str());
    let file = File::parse_str(file.as_str());

    let mut cfgs = Vec::new();
    if args.extended {
        cfgs.push("deno_bindgen_rust_string");
    }

    let dylib_path = Cargo::build(&metadata.pkg_name, args.release, cfgs);

    let file_name;
    let out_path;

    if let Some(output) = &args.output {
        file_name = output
            .file_name()
            .expect("invalid output file name")
            .to_str()
            .expect("unknown utf8 character on file name")
            .to_string();
        let mut _out_path = output.clone();
        _out_path.pop();
        out_path = _out_path;
    } else {
        file_name = format!("lib{}.ts", metadata.lib_name);
        out_path = metadata.pkg_path.join("dist");
        std::fs::create_dir_all(&out_path).expect("failed to create `dist` dir");
    }

    let opts = args.to_codegen_opts(
        file_name.to_string(),
        metadata.strip_workspace_path(&dylib_path),
    );
    let module = TsModule::new(file, &opts);

    if args.inline {
        let module = module.generate_single(&opts);
        let mut module_file =
            std::fs::File::create(out_path.join(file_name)).expect("failed to create module file");
        module_file
            .write(module.as_bytes())
            .expect("failed to write into module file");
    } else {
        let (module, type_defs) = module.generate_multiple(&opts, "rust_type.ts");

        let mut module_file =
            std::fs::File::create(out_path.join(file_name)).expect("failed to create module file");
        module_file
            .write(module.as_bytes())
            .expect("failed to write into module file");

        let mut typedefs_file = std::fs::File::create(out_path.join("rust_type.ts"))
            .expect("failed to create typedefs file");
        typedefs_file
            .write(type_defs.as_bytes())
            .expect("failed to write into typedefs file");
    }

    println!("{} ready", metadata.pkg_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let metadata = Cargo::get_metadata();
        let content = Cargo::expand(metadata.pkg_name.as_str());

        print!("{content}");

        let content = File::parse_str(content.as_str());
        dbg!(&content);
    }
}
