use deno_bindgen2_common::File;

use crate::cargo::Cargo;

pub struct Parser {}

impl Parser {
    pub fn parse() -> File {
        let pkg_name = Cargo::get_pkg_name(Some("../test"));
        let content = Cargo::expand(pkg_name.as_str());

        print!("{content}");

        let content = File::parse_str(content.as_str());

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_file() {
        let content = Parser::parse();

        dbg!(content.items);
    }
}
