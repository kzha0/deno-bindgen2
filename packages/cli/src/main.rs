use deno_bindgen2_common::File;

// mod bootstrap;
mod cargo;
mod interface;
mod parse;

// mod utils;
// #[macro_use]
// use utils::*;

use cargo::Cargo;

pub type Result<T> = std::result::Result<T, ()>;

// planned module layout
// main - entrypoint
// cargo - cargo commands
// codegen - code generation? (should'nt this be moved to /common?)
// interface - CLI options and prompt


/// guided prompt mode
/// parses source code to check modules
// fn interactive(args: &mut interface::Cli) -> Result<()> {
//     let output = inquire::Text::new(
//         "Where should the output file be emitted? (leave blank to output to
// console)",     )
//     .prompt_skippable()?;

//     if let Some(output) = output {
//         if output.chars().count() > 0 {
//             args.with_output(output.as_str());
//         }
//     }

//     // this command should be invocable in autogenerate mode
//     // without needing to prompt users which modules to include
//     let source_file = cargo::cargo_expand()?;

//     parse::parse_source(source_file)?;


//     // [!todo] prelude injection
//     // use `rustc --extern `
//     // handle insertion of prelude module containing utility module symbols
//     // https://doc.rust-lang.org/reference/names/preludes.html#extern-prelude

//     Ok(())
// }


fn main() -> Result<()> {
    // let mut args: interface::Cli = clap::Parser::parse();

    // if args.try_interactive() {
    //     interactive(&mut args)?;
    // }

    Cargo::precheck();
    let pkg_name = Cargo::get_pkg_name(
        #[cfg(debug_assertions)]
        None,
    );

    let content = Cargo::expand(pkg_name.as_str());

    let content = File::parse_str(content.as_str());




    Ok(())
}
