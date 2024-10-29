pub use proc_macro2::{TokenStream, Span};
pub use quote::{format_ident, quote, ToTokens};
pub use syn::parse::discouraged::Speculative;
pub use syn::parse::{Parse, ParseStream};
pub use syn::spanned::Spanned;
pub use syn::token::{Bracket, Paren};
pub use syn::visit_mut::VisitMut;
pub use syn::{
    braced, bracketed, parenthesized, Attribute, Error, Expr, Ident, Lifetime, LitInt,
    LitStr, Pat, Result, Token, Visibility,
};

pub use crate::{dbg_assert, dbg_quote, parse_quote};

#[macro_export]
macro_rules! parse_quote {
    ( $ty:ty, $($tt:tt)* ) => {
        syn::parse2::<$ty>(quote::quote!{ $($tt)* })
            .map_err(|err| panic!("{err:#?}"))
            .unwrap()
    };
}

#[macro_export]
macro_rules! dbg_quote {
    ( $($tt:tt)* ) => {
        dbg!(parse_quote!($($tt)*))
    };
}

#[macro_export]
macro_rules! dbg_assert {
    ( $expected:expr_2021, $actual:expr_2021 ) => {
        dbg!($actual);
        assert_eq!($expected, $actual)
    };
}
