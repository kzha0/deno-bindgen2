use std::path::PathBuf;

use deno::Generator;

use crate::*;

mod deno;

pub struct Options {
    pub target:           Target,
    pub out:              Option<PathBuf>,
    pub local_dylib_path: PathBuf,
    pub lazy_init:        bool,
}

pub enum Target {
    Deno,
}

pub fn generate(items: &'static [RawItem], opt: Options) -> std::io::Result<()> {
    let mut generator = match opt.target {
        Target::Deno => deno::Generator::new(items, &opt.local_dylib_path, opt.lazy_init),
    };

    if let Some(out) = opt.out {
        let out = std::fs::File::create(out)?;
        Generator::generate(generator, out)?;
        return Ok(());
    };

    let out = std::io::stdout();
    Generator::generate(generator, out)
}
